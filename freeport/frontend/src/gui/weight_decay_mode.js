export const WeightDecayMode = {
    /// Simple (Immediatelly decay to 0)
    Simple: 0,
    /// Complex with weight decay factor 0.3
    FastDecay: 1,
    /// Complex with weight decay factor 0.5
    MediumDecay: 2,
    /// Complex with weight decay factor 0.7
    SlowDecay: 3,
    /// Compound (No decay)
    Compound: 4,
};