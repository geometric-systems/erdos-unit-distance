use crate::MultiquadraticConfig;
use crate::algebraic::field::{
    FieldElement, MultiQuadraticField, rational_from_string, rational_to_string,
};
use crate::algebraic::lattice::project_to_plane_embedding;
use crate::algebraic::units::{find_prime_element, generate_unit_modulus_elements};
use crate::backend::BackendProvenance;
use crate::certificate::FloatingAuditReport;
use crate::error::{GenerationError, VerificationError};
use crate::numeric::{i64_to_f64_verification, rounded_f64_to_i64};
use crate::utils::find_unit_distance_edges;
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};

pub const ALGEBRAIC_KEY_DENOMINATOR: i64 = 1_000_000_000;
const WINDOW_TOLERANCE: f64 = 1e-7;
const PROJECTION_TOLERANCE: f64 = 1e-7;

#[derive(Clone, Debug)]
pub struct Section2Input {
    pub config: MultiquadraticConfig,
    pub translations: Vec<FieldElement>,
    pub window_radius: f64,
    pub projection_index: usize,
    pub target_count: usize,
}

#[derive(Clone, Debug)]
pub struct Section2Output {
    pub algebraic_candidates: Vec<FieldElement>,
    pub projected_points: Vec<[f64; 2]>,
    pub construction_edges: Vec<ConstructionEdgeCertificate>,
    pub certificate: Section2Certificate,
}

#[derive(Clone, Debug)]
struct CandidateRecord {
    element: FieldElement,
    key: AlgebraicElementKey,
    path: Vec<SignedTranslationStep>,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct AlgebraicElementKey {
    pub coeffs: Vec<String>,
}

impl AlgebraicElementKey {
    pub fn from_element(element: &FieldElement) -> Result<Self, VerificationError> {
        Ok(Self {
            coeffs: element.coeffs().iter().map(rational_to_string).collect(),
        })
    }

    pub fn from_element_with_denominator(
        element: &FieldElement,
        _denominator: i64,
    ) -> Result<Self, VerificationError> {
        Self::from_element(element)
    }

    pub fn to_element(
        &self,
        field: &MultiQuadraticField,
    ) -> Result<FieldElement, VerificationError> {
        if self.coeffs.len() != field.degree() {
            return Err(VerificationError::AlgebraicKeyDimensionMismatch {
                expected: field.degree(),
                actual: self.coeffs.len(),
            });
        }
        FieldElement::try_from_coeffs(
            self.coeffs
                .iter()
                .map(|coeff| rational_from_string(coeff))
                .collect::<Result<Vec<_>, VerificationError>>()?,
            field,
        )
        .map_err(|err| VerificationError::InvalidConstruction {
            reason: err.to_string(),
        })
    }

    pub fn zero(field: &MultiQuadraticField) -> Self {
        Self {
            coeffs: vec!["0".to_string(); field.degree()],
        }
    }

    pub fn one(field: &MultiQuadraticField) -> Self {
        let mut coeffs = vec!["0".to_string(); field.degree()];
        coeffs[0] = "1".to_string();
        Self { coeffs }
    }

    pub fn add(
        &self,
        other: &Self,
        field: &MultiQuadraticField,
    ) -> Result<Self, VerificationError> {
        Self::from_element(&self.to_element(field)?.add(&other.to_element(field)?))
    }

    pub fn sub(
        &self,
        other: &Self,
        field: &MultiQuadraticField,
    ) -> Result<Self, VerificationError> {
        Self::from_element(&self.to_element(field)?.sub(&other.to_element(field)?))
    }

    pub fn mul(
        &self,
        other: &Self,
        field: &MultiQuadraticField,
    ) -> Result<Self, VerificationError> {
        Self::from_element(
            &self
                .to_element(field)?
                .mul(&other.to_element(field)?, field),
        )
    }

    pub fn conjugate_gen(
        &self,
        gen_idx: usize,
        field: &MultiQuadraticField,
    ) -> Result<Self, VerificationError> {
        self.validate(field)?;
        Self::from_element(&self.to_element(field)?.conjugate_gen(gen_idx))
    }

    pub fn complex_conjugate(
        &self,
        field: &MultiQuadraticField,
    ) -> Result<Self, VerificationError> {
        match field.imaginary_generator_index() {
            Some(index) => self.conjugate_gen(index, field),
            None => Ok(self.clone()),
        }
    }

    pub fn is_one(&self, field: &MultiQuadraticField) -> Result<bool, VerificationError> {
        Ok(self == &Self::one(field))
    }

    fn validate(&self, field: &MultiQuadraticField) -> Result<(), VerificationError> {
        if self.coeffs.len() != field.degree() {
            return Err(VerificationError::AlgebraicKeyDimensionMismatch {
                expected: field.degree(),
                actual: self.coeffs.len(),
            });
        }
        Ok(())
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct TranslationCertificate {
    pub element: AlgebraicElementKey,
    pub norm: AlgebraicElementKey,
    pub max_modulus_error: f64,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct CandidateCertificate {
    pub element: AlgebraicElementKey,
    pub path: Vec<SignedTranslationStep>,
    pub max_embedding_norm: f64,
    pub projection: [f64; 2],
    pub projection_key: [i64; 2],
    pub dedup_key: [i64; 2],
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedTranslationStep {
    pub translation_index: usize,
    pub sign: i8,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructionEdgeCertificate {
    pub endpoints: (usize, usize),
    pub translation_index: usize,
    pub sign: i8,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Section2Certificate {
    pub generators: Vec<i64>,
    pub split_prime: i64,
    pub k: usize,
    pub max_prime_exponent: usize,
    pub window_radius: f64,
    pub projection_index: usize,
    pub target_count: usize,
    pub backend: BackendProvenance,
    pub translations: Vec<TranslationCertificate>,
    pub candidates: Vec<CandidateCertificate>,
    pub construction_edges: Vec<ConstructionEdgeCertificate>,
    pub audit: FloatingAuditReport,
}

impl Section2Certificate {
    pub fn verify(&self) -> Result<(), VerificationError> {
        let config = MultiquadraticConfig::new(
            self.generators_without_auto_imaginary(),
            self.split_prime,
            self.k,
        )
        .map_err(|err| VerificationError::InvalidConstruction {
            reason: err.to_string(),
        })?
        .with_prime_search_limit(self.max_prime_exponent)
        .map_err(|err| VerificationError::InvalidConstruction {
            reason: err.to_string(),
        })?;
        let field = MultiQuadraticField::try_new(config.generators()).map_err(|err| {
            VerificationError::InvalidConstruction {
                reason: err.to_string(),
            }
        })?;

        self.translations
            .iter()
            .enumerate()
            .try_for_each(|(index, translation)| {
                verify_translation_certificate(index, translation, &field)
            })?;
        let translation_keys = self
            .translations
            .iter()
            .map(|translation| translation.element.clone())
            .collect::<Vec<_>>();

        if self.candidates.len() != self.target_count {
            return Err(VerificationError::PointCountMismatch {
                expected: self.target_count,
                actual: self.candidates.len(),
            });
        }

        let points = self
            .candidates
            .iter()
            .enumerate()
            .map(|(index, candidate)| {
                verify_candidate_certificate(index, candidate, self, &field, &translation_keys)
            })
            .collect::<Result<Vec<_>, _>>()?;

        verify_construction_edges(&self.candidates, &translation_keys, self)?;
        verify_unit_distance_edges(&points, &self.audit)
    }

    pub fn projected_points(&self) -> Vec<[f64; 2]> {
        self.candidates
            .iter()
            .map(|candidate| candidate.projection)
            .collect()
    }

    fn generators_without_auto_imaginary(&self) -> Vec<i64> {
        self.generators
            .iter()
            .copied()
            .filter(|&generator| generator != -1)
            .collect()
    }
}

pub fn generate_section2(input: Section2Input) -> Result<Section2Output, GenerationError> {
    let field = MultiQuadraticField::try_new(input.config.generators())?;
    let max_points = input.target_count * input.config.candidate_multiplier();
    let candidate_denominator = candidate_denominator(&input.config, &field)?;
    let translation_keys = input
        .translations
        .iter()
        .map(|translation| {
            AlgebraicElementKey::from_element_with_denominator(translation, candidate_denominator)
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| GenerationError::InvalidSearchParameter {
            parameter: "translation_key",
            reason: "could not quantize translation key",
        })?;
    let algebraic_candidates = search_candidates(
        &field,
        &input.translations,
        &translation_keys,
        input.window_radius,
        max_points,
    )?;
    let projected_candidates = project_and_deduplicate_candidates(
        &algebraic_candidates,
        &field,
        input.projection_index,
        input.config.projection_tolerance(),
    )?;

    if projected_candidates.len() < input.target_count {
        return Err(GenerationError::InsufficientPoints {
            requested: input.target_count,
            found: projected_candidates.len(),
        });
    }

    let selected_indices = select_dense_projected_indices(
        projected_candidates
            .iter()
            .map(|(_, point)| *point)
            .collect::<Vec<_>>()
            .as_slice(),
        input.target_count,
        input.config.unit_distance_tolerance(),
    );
    let selected = selected_indices
        .into_iter()
        .map(|index| projected_candidates[index].clone())
        .collect::<Vec<_>>();
    let selected_candidates = selected
        .iter()
        .map(|(candidate, _)| candidate.clone())
        .collect::<Vec<_>>();
    let projected_points = selected.iter().map(|(_, point)| *point).collect::<Vec<_>>();
    let construction_edges =
        construction_edges_from_candidates(&selected_candidates, &translation_keys, &field)?;
    let unit_distance_edges =
        find_unit_distance_edges(&projected_points, input.config.unit_distance_tolerance());
    let candidate_certificates = selected_candidates
        .iter()
        .zip(&projected_points)
        .map(|(candidate, &projection)| {
            Ok(CandidateCertificate {
                element: candidate.key.clone(),
                path: candidate.path.clone(),
                max_embedding_norm: max_embedding_norm(&candidate.element, &field),
                projection,
                projection_key: projection_key(projection).map_err(|_| {
                    GenerationError::InvalidSearchParameter {
                        parameter: "projection_key",
                        reason: "could not quantize projected point",
                    }
                })?,
                dedup_key: dedup_key(projection, input.config.projection_tolerance()).map_err(
                    |_| GenerationError::InvalidSearchParameter {
                        parameter: "dedup_key",
                        reason: "could not quantize projected point",
                    },
                )?,
            })
        })
        .collect::<Result<Vec<_>, GenerationError>>()?;

    Ok(Section2Output {
        algebraic_candidates: selected_candidates
            .iter()
            .map(|candidate| candidate.element.clone())
            .collect(),
        projected_points,
        construction_edges: construction_edges.clone(),
        certificate: Section2Certificate {
            generators: input.config.generators().to_vec(),
            split_prime: input.config.split_prime(),
            k: input.config.k(),
            max_prime_exponent: input.config.max_prime_exponent(),
            window_radius: input.window_radius,
            projection_index: input.projection_index,
            target_count: input.target_count,
            backend: BackendProvenance::native_multiquadratic(),
            translations: {
                let denominator = translation_denominator(&input.config, &field)?;
                input
                    .translations
                    .iter()
                    .map(|translation| translation_certificate(translation, &field, denominator))
                    .collect::<Result<Vec<_>, _>>()?
            },
            candidates: candidate_certificates,
            construction_edges,
            audit: FloatingAuditReport {
                tolerance: input.config.unit_distance_tolerance(),
                edges: unit_distance_edges,
            },
        },
    })
}

pub fn generate_native_multiquadratic_section2(
    config: &MultiquadraticConfig,
    target_count: usize,
) -> Result<Section2Output, GenerationError> {
    let field = MultiQuadraticField::try_new(config.generators())?;

    (0..config.max_radius_attempts())
        .map(|idx| {
            i32::try_from(idx)
                .map(|idx| config.initial_radius() * config.radius_growth().powi(idx))
                .map_err(|_| GenerationError::InvalidSearchParameter {
                    parameter: "radius_attempt",
                    reason: "radius attempt index does not fit i32",
                })
        })
        .map(|window_radius| {
            let window_radius = window_radius?;
            generate_unit_modulus_elements(
                &field,
                config.split_prime(),
                config.k(),
                config.max_prime_exponent(),
            )
            .and_then(|translations| {
                generate_section2(Section2Input {
                    config: config.clone(),
                    translations,
                    window_radius,
                    projection_index: 0,
                    target_count,
                })
            })
        })
        .find_map(|result| match result {
            Ok(output) if output.projected_points.len() >= target_count => Some(Ok(output)),
            Ok(_) => None,
            Err(err) => Some(Err(err)),
        })
        .unwrap_or(Err(GenerationError::InsufficientPoints {
            requested: target_count,
            found: 0,
        }))
}

fn search_candidates(
    field: &MultiQuadraticField,
    translations: &[FieldElement],
    translation_keys: &[AlgebraicElementKey],
    window_radius: f64,
    max_points: usize,
) -> Result<Vec<CandidateRecord>, GenerationError> {
    let mut queue = VecDeque::new();
    let mut visited = Vec::new();
    let mut seen = HashSet::new();
    let origin_element = FieldElement::zero(field);
    let origin_key = AlgebraicElementKey::zero(field);
    let origin = CandidateRecord {
        element: origin_element,
        key: origin_key,
        path: Vec::new(),
    };
    queue.push_back(origin.clone());
    seen.insert(origin.key.clone());
    visited.push(origin);

    while let Some(curr) = queue.pop_front() {
        if visited.len() >= max_points {
            break;
        }

        for (translation_index, translation) in translations.iter().enumerate() {
            for (sign, next) in [
                (1_i8, curr.element.add(translation)),
                (-1_i8, curr.element.sub(translation)),
            ] {
                if max_embedding_norm(&next, field) > window_radius + WINDOW_TOLERANCE {
                    continue;
                }

                let next_key = match sign {
                    1 => curr.key.add(&translation_keys[translation_index], field),
                    -1 => curr.key.sub(&translation_keys[translation_index], field),
                    _ => unreachable!(),
                }
                .map_err(|_| GenerationError::InvalidSearchParameter {
                    parameter: "candidate_key",
                    reason: "generated exact candidate key overflowed",
                })?;

                if seen.insert(next_key.clone()) {
                    let mut path = curr.path.clone();
                    path.push(SignedTranslationStep {
                        translation_index,
                        sign,
                    });
                    let record = CandidateRecord {
                        element: next,
                        key: next_key,
                        path,
                    };
                    visited.push(record.clone());
                    queue.push_back(record);
                    if visited.len() >= max_points {
                        break;
                    }
                }
            }
            if visited.len() >= max_points {
                break;
            }
        }
    }

    Ok(visited)
}

fn project_and_deduplicate_candidates(
    candidates: &[CandidateRecord],
    field: &MultiQuadraticField,
    projection_index: usize,
    tolerance: f64,
) -> Result<Vec<(CandidateRecord, [f64; 2])>, GenerationError> {
    let alpha = FieldElement::one(field);
    candidates
        .iter()
        .try_fold(Vec::new(), |mut projected, candidate| {
            let point =
                project_to_plane_embedding(&candidate.element, field, &alpha, projection_index)?;
            if !projected
                .iter()
                .any(|(_, existing)| point_distance(point, *existing) < tolerance)
            {
                projected.push((candidate.clone(), point));
            }
            Ok(projected)
        })
}

fn select_dense_projected_indices(
    points: &[[f64; 2]],
    target_count: usize,
    tolerance: f64,
) -> Vec<usize> {
    let n_initial = points.len();
    if n_initial <= target_count {
        return (0..n_initial).collect();
    }

    let mut adj = vec![Vec::new(); n_initial];
    let mut degrees = vec![0; n_initial];
    let mut active = vec![true; n_initial];
    find_unit_distance_edges(points, tolerance)
        .into_iter()
        .for_each(|(i, j)| {
            adj[i].push(j);
            adj[j].push(i);
            degrees[i] += 1;
            degrees[j] += 1;
        });

    let mut active_count = n_initial;
    while active_count > target_count {
        let Some(min_idx) = (0..n_initial)
            .filter(|&i| active[i])
            .min_by_key(|&i| degrees[i])
        else {
            break;
        };

        active[min_idx] = false;
        active_count -= 1;
        let neighbors = adj[min_idx]
            .iter()
            .copied()
            .filter(|&neighbor| active[neighbor] && degrees[neighbor] > 0)
            .collect::<Vec<_>>();
        neighbors
            .into_iter()
            .for_each(|neighbor| degrees[neighbor] -= 1);
    }

    active
        .into_iter()
        .enumerate()
        .filter_map(|(index, is_active)| is_active.then_some(index))
        .collect()
}

fn translation_certificate(
    translation: &FieldElement,
    field: &MultiQuadraticField,
    denominator: i64,
) -> Result<TranslationCertificate, GenerationError> {
    let element = AlgebraicElementKey::from_element_with_denominator(translation, denominator)
        .map_err(|_| GenerationError::InvalidSearchParameter {
            parameter: "translation_certificate",
            reason: "could not quantize translation element certificate",
        })?;
    let norm = element
        .complex_conjugate(field)
        .and_then(|conjugate| element.mul(&conjugate, field))
        .map_err(|_| GenerationError::InvalidSearchParameter {
            parameter: "translation_certificate",
            reason: "could not build exact translation norm certificate",
        })?;
    Ok(TranslationCertificate {
        element,
        norm,
        max_modulus_error: translation
            .embeddings(field)
            .into_iter()
            .map(|embedding| (embedding.norm() - 1.0).abs())
            .fold(0.0, f64::max),
    })
}

fn translation_denominator(
    config: &MultiquadraticConfig,
    field: &MultiQuadraticField,
) -> Result<i64, GenerationError> {
    let (_, y) = find_prime_element(field, config.split_prime(), config.max_prime_exponent())?;
    let real_degree = 1_usize << (field.generator_count() - 1);
    let exponent = y * config.k() * real_degree;
    if !exponent.is_multiple_of(2) {
        return Err(GenerationError::InvalidSearchParameter {
            parameter: "unit_denominator",
            reason: "expected an even unit scaling exponent",
        });
    }
    (0..(exponent / 2)).try_fold(1_i64, |denominator, _| {
        denominator.checked_mul(config.split_prime()).ok_or(
            GenerationError::InvalidSearchParameter {
                parameter: "unit_denominator",
                reason: "unit denominator overflowed i64",
            },
        )
    })
}

fn candidate_denominator(
    config: &MultiquadraticConfig,
    field: &MultiQuadraticField,
) -> Result<i64, GenerationError> {
    translation_denominator(config, field)
}

fn verify_translation_certificate(
    index: usize,
    certificate: &TranslationCertificate,
    field: &MultiQuadraticField,
) -> Result<(), VerificationError> {
    let exact_norm = certificate
        .element
        .mul(&certificate.element.complex_conjugate(field)?, field)?;
    let has_exact_norm_one = exact_norm.is_one(field)?;
    let norm_certificate_matches = exact_norm == certificate.norm;

    let translation = certificate.element.to_element(field)?;
    let has_unit_embeddings = translation
        .embeddings(field)
        .into_iter()
        .all(|embedding| (embedding.norm() - 1.0).abs() <= 1e-5);

    if has_exact_norm_one && has_unit_embeddings && norm_certificate_matches {
        Ok(())
    } else {
        Err(VerificationError::TranslationNotNormOne { index })
    }
}

fn verify_candidate_certificate(
    index: usize,
    candidate: &CandidateCertificate,
    certificate: &Section2Certificate,
    field: &MultiQuadraticField,
    translation_keys: &[AlgebraicElementKey],
) -> Result<[f64; 2], VerificationError> {
    let replayed = replay_candidate_path(index, &candidate.path, field, translation_keys)?;
    if replayed != candidate.element {
        return Err(VerificationError::CandidatePathMismatch { index });
    }

    let element = candidate.element.to_element(field)?;
    let max_norm = max_embedding_norm(&element, field);
    if max_norm > certificate.window_radius + WINDOW_TOLERANCE {
        return Err(VerificationError::CandidateOutsideWindow {
            index,
            radius: certificate.window_radius.to_string(),
            max_embedding_norm: max_norm.to_string(),
        });
    }

    let alpha = FieldElement::one(field);
    let projection =
        project_to_plane_embedding(&element, field, &alpha, certificate.projection_index).map_err(
            |err| VerificationError::InvalidConstruction {
                reason: err.to_string(),
            },
        )?;
    if point_distance(projection, candidate.projection) > PROJECTION_TOLERANCE {
        return Err(VerificationError::ProjectionMismatch { index });
    }

    let expected_projection_key = projection_key(candidate.projection)?;
    if expected_projection_key != candidate.projection_key {
        return Err(VerificationError::QuantizedProjectionMismatch {
            index,
            expected: expected_projection_key,
            actual: candidate.projection_key,
        });
    }

    let expected_dedup_key = dedup_key(candidate.projection, PROJECTION_TOLERANCE)?;
    if expected_dedup_key != candidate.dedup_key {
        return Err(VerificationError::QuantizedProjectionMismatch {
            index,
            expected: expected_dedup_key,
            actual: candidate.dedup_key,
        });
    }

    Ok(candidate.projection)
}

fn replay_candidate_path(
    candidate_index: usize,
    path: &[SignedTranslationStep],
    field: &MultiQuadraticField,
    translation_keys: &[AlgebraicElementKey],
) -> Result<AlgebraicElementKey, VerificationError> {
    path.iter()
        .try_fold(AlgebraicElementKey::zero(field), |candidate, step| {
            let Some(translation) = translation_keys.get(step.translation_index) else {
                return Err(VerificationError::CandidatePathTranslationOutOfBounds {
                    candidate_index,
                    translation_index: step.translation_index,
                    translation_count: translation_keys.len(),
                });
            };
            match step.sign {
                1 => candidate.add(translation, field),
                -1 => candidate.sub(translation, field),
                _ => Err(VerificationError::InvalidConstruction {
                    reason: format!(
                        "candidate {candidate_index} has invalid path sign {}",
                        step.sign
                    ),
                }),
            }
        })
}

fn construction_edges_from_candidates(
    candidates: &[CandidateRecord],
    translation_keys: &[AlgebraicElementKey],
    field: &MultiQuadraticField,
) -> Result<Vec<ConstructionEdgeCertificate>, GenerationError> {
    let index_by_key = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| (candidate.key.clone(), index))
        .collect::<HashMap<_, _>>();

    Ok(candidates
        .iter()
        .enumerate()
        .flat_map(|(source_index, candidate)| {
            translation_keys.iter().enumerate().filter_map({
                let index_by_key = &index_by_key;
                move |(translation_index, translation)| {
                    candidate
                        .key
                        .add(translation, field)
                        .ok()
                        .and_then(|target_key| index_by_key.get(&target_key).copied())
                        .filter(|&target_index| source_index < target_index)
                        .map(|target_index| ConstructionEdgeCertificate {
                            endpoints: (source_index, target_index),
                            translation_index,
                            sign: 1,
                        })
                }
            })
        })
        .collect())
}

fn verify_construction_edges(
    candidates: &[CandidateCertificate],
    translation_keys: &[AlgebraicElementKey],
    certificate: &Section2Certificate,
) -> Result<(), VerificationError> {
    let field = MultiQuadraticField::try_new(&certificate.generators).map_err(|err| {
        VerificationError::InvalidConstruction {
            reason: err.to_string(),
        }
    })?;
    certificate
        .construction_edges
        .iter()
        .enumerate()
        .try_for_each(|(edge_index, edge)| {
            if edge.endpoints.0 == edge.endpoints.1 {
                return Err(VerificationError::SelfEdge {
                    index: edge.endpoints.0,
                });
            }
            if edge.endpoints.0 >= candidates.len() || edge.endpoints.1 >= candidates.len() {
                return Err(VerificationError::EdgeIndexOutOfBounds {
                    edge: edge.endpoints,
                    point_count: candidates.len(),
                });
            }
            let Some(translation) = translation_keys.get(edge.translation_index) else {
                return Err(VerificationError::EdgeProvenanceMismatch { edge_index });
            };
            let diff = candidates[edge.endpoints.1]
                .element
                .sub(&candidates[edge.endpoints.0].element, &field)?;
            let expected = match edge.sign {
                1 => translation.clone(),
                -1 => AlgebraicElementKey::zero(&field).sub(translation, &field)?,
                _ => {
                    return Err(VerificationError::EdgeProvenanceMismatch { edge_index });
                }
            };
            if diff == expected {
                Ok(())
            } else {
                Err(VerificationError::EdgeProvenanceMismatch { edge_index })
            }
        })?;

    Ok(())
}

fn verify_unit_distance_edges(
    points: &[[f64; 2]],
    certificate: &FloatingAuditReport,
) -> Result<(), VerificationError> {
    certificate.edges.iter().try_for_each(|&edge| {
        if edge.0 == edge.1 {
            Err(VerificationError::SelfEdge { index: edge.0 })
        } else if edge.0 >= points.len() || edge.1 >= points.len() {
            Err(VerificationError::EdgeIndexOutOfBounds {
                edge,
                point_count: points.len(),
            })
        } else {
            Ok(())
        }
    })?;

    let mut expected = find_unit_distance_edges(points, certificate.tolerance);
    let mut actual = certificate.edges.clone();
    expected.sort_unstable();
    actual.sort_unstable();
    if expected == actual {
        Ok(())
    } else {
        Err(VerificationError::EdgeSetMismatch { expected, actual })
    }
}

fn max_embedding_norm(element: &FieldElement, field: &MultiQuadraticField) -> f64 {
    element
        .embeddings(field)
        .into_iter()
        .map(|embedding| embedding.norm())
        .fold(0.0, f64::max)
}

fn point_distance(left: [f64; 2], right: [f64; 2]) -> f64 {
    let dx = left[0] - right[0];
    let dy = left[1] - right[1];
    (dx * dx + dy * dy).sqrt()
}

fn projection_key(point: [f64; 2]) -> Result<[i64; 2], VerificationError> {
    let scale = i64_to_f64_verification(ALGEBRAIC_KEY_DENOMINATOR)?;
    Ok([
        rounded_f64_to_i64(point[0] * scale)?,
        rounded_f64_to_i64(point[1] * scale)?,
    ])
}

fn dedup_key(point: [f64; 2], tolerance: f64) -> Result<[i64; 2], VerificationError> {
    Ok([
        rounded_f64_to_i64(point[0] / tolerance)?,
        rounded_f64_to_i64(point[1] / tolerance)?,
    ])
}
