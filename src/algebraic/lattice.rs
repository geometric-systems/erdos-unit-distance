use crate::algebraic::field::{FieldElement, MultiQuadraticField};
use crate::error::GenerationError;

/// Computes the Minkowski embedding of a field element.
/// Since K is a CM field of degree N = 2^m, there are N/2 pairs of complex conjugate embeddings.
/// We select one embedding from each pair (specifically where the imaginary unit generator is positive/positive-real-sign).
/// Each selected complex embedding gives 2 real coordinates, yielding a vector of N real coordinates in R^N.
pub fn minkowski_embedding(
    elem: &FieldElement,
    field: &MultiQuadraticField,
    alpha: &FieldElement,
) -> Result<Vec<f64>, GenerationError> {
    let i_idx = field
        .imaginary_generator_index()
        .ok_or(GenerationError::MissingImaginaryGenerator)?;
    let embs = elem.embeddings(field);
    let alpha_embs = alpha.embeddings(field);

    Ok((0..field.degree())
        .filter(|s| ((s >> i_idx) & 1) == 0)
        .flat_map(|s| {
            let val = embs[s];
            let alpha_val = alpha_embs[s].re.abs();
            let scale = if alpha_val > 1e-15 {
                alpha_val.sqrt()
            } else {
                1.0
            };
            let scaled = val / scale;
            [scaled.re, scaled.im]
        })
        .collect())
}

/// Projects a field element to a 2D point in the plane.
/// This corresponds to one complex embedding (e.g. the first one s = 0) scaled by sqrt(|alpha_0|).
pub fn project_to_plane(
    elem: &FieldElement,
    field: &MultiQuadraticField,
    alpha: &FieldElement,
) -> Result<[f64; 2], GenerationError> {
    project_to_plane_embedding(elem, field, alpha, 0)
}

/// Projects a field element to a 2D point using a selected complex embedding.
pub fn project_to_plane_embedding(
    elem: &FieldElement,
    field: &MultiQuadraticField,
    alpha: &FieldElement,
    projection_index: usize,
) -> Result<[f64; 2], GenerationError> {
    field
        .imaginary_generator_index()
        .ok_or(GenerationError::MissingImaginaryGenerator)?;
    let embs = elem.embeddings(field);
    let alpha_embs = alpha.embeddings(field);
    if projection_index >= field.degree() {
        return Err(GenerationError::InvalidSearchParameter {
            parameter: "projection_index",
            reason: "expected an embedding index within the field degree",
        });
    }

    let val = embs[projection_index];
    let alpha_val = alpha_embs[projection_index].re.abs();
    let scale = if alpha_val > 1e-15 {
        alpha_val.sqrt()
    } else {
        1.0
    };

    let projected = val / scale;
    Ok([projected.re, projected.im])
}
