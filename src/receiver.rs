use anyhow::Result;
use basic_dsp::CrossCorrelationOps;
use num_complex::Complex32;
use tap::Pipe;

use crate::Signal;

#[optargs::optfn]
pub fn decode(mut samples: Vec<Complex32>, guard_bands: Option<bool>) -> Vec<u8> {
    let guard_bands = guard_bands.unwrap_or_else(|| false);

    // hardcode the delay from the signal
    // we should do this with xcorr but that's a bit slow, unfortunately
    let samples = samples.split_off(9);

    // Create an iterator over all the chunks we received
    // We need to pad the final chunk because we added zeros to fill in the gaps made by guardbands
    let mut chunks = split_into_chunks(samples);
    let mut chunk_iter = chunks.iter_mut();

    // Pull out the locking signals and preamble signals
    let (locking, pre1, pre2, pre3) = (
        chunk_iter.next(),
        chunk_iter.next(),
        chunk_iter.next(),
        chunk_iter.next(),
    );

    // Create a buffer to dump raw samples into
    let mut out_stream = Vec::new();

    // Eat through all the chunks, decoding them and pushing them into our stream
    while let Some(next_chunk) = chunk_iter.next() {
        let unprefixed = unprefix_block(next_chunk);
        decode_block(unprefixed, guard_bands, &mut out_stream);
    }

    // Convert our vec of Complex numbers into a vec of bytes
    demodulate(out_stream)
}

/// Remove the cyclic prefix and then write into the buffer
pub fn unprefix_block(input: &[Complex32; 80]) -> [Complex32; 64] {
    let mut output = [Complex32::default(); 64];

    input
        .iter()
        .skip(16)
        .zip(output.iter_mut())
        .for_each(|(i, o)| *o = *i);

    output.fft();

    output
}

pub fn decode_block(input: [Complex32; 64], guard_bands: bool, output: &mut Vec<Complex32>) {
    let mut input_iter = std::array::IntoIter::new(input).enumerate();

    while let Some((i, next)) = input_iter.next() {
        match (
            guard_bands,
            // Write 0s at the start, end, and at the dc offset
            i >= 59 || i <= 6 || i == 32,
            // Write 1s at intermediate guardbands
            i == 6 || i == 25 || i == 39 || i == 58,
        ) {
            // If guardbands are enabled and we ran into one, just don't add this
            (true, true, _) => {
                // assert_eq!(next.re, 0.0)
            }
            (true, _, true) => {
                // assert_eq!(next.re, 1.0)
            }

            // If the sample is not on a guardband, then push it into the output
            (_, _, _) => output.push(next),
        };
    }
    // dbg!(output.len());
}

pub fn demodulate(stream: Vec<Complex32>) -> Vec<u8> {
    let mut out = Vec::new();

    let sliced = stream.into_boxed_slice();
    let (out_chunks, remainder) = sliced.as_chunks::<8>();
    dbg!(remainder);

    for chunk in out_chunks {
        let mut bools = [false; 8];
        for (sample, slot) in chunk.iter().zip(bools.iter_mut()) {
            // This is BPSK
            // we'll want a more complex modulation scheme
            *slot = sample.re > 0.0;
        }
        out.push(crate::utils::bools_to_u8(bools));
    }

    out
}

pub fn split_into_chunks(samples: Vec<Complex32>) -> Vec<[Complex32; 80]> {
    let mut samples = samples.into_boxed_slice();

    let (chunks, remainder) = samples.as_chunks_mut::<80>();
    let mut chunk_vec = Vec::from(chunks);
    if remainder.len() > 0 {
        chunk_vec.push(pad_chunk(remainder));
    }

    chunk_vec
}

/// split into chunks
pub fn pad_chunk(remainder: &[Complex32]) -> [Complex32; 80] {
    dbg!(remainder.len());
    let mut out = [Complex32::default(); 80];
    for (sample, slot) in remainder.iter().zip(out.iter_mut()) {
        *slot = *sample;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
}
