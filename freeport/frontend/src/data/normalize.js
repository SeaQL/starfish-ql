/// Normalize entries in data by min-max normalization.
/// Use the returned array of normalized entries if you wish to avoid side effects.
export const normalizeData = (
    data,
    getEntries = (data) => Array.from(data),
    insertEntries = (data, normalizedEntries) => {},
    {
        newMin,
        newMax,
    } = {},
) => {
    const entries = getEntries(data);

    const min = Math.min(...entries);
    const max = Math.max(...entries);
    let range = max - min;
    const denom = range !== 0 ? range : 1;

    if (newMin !== undefined && newMax === undefined) {
        newMax = newMin + range;
    } else if (newMin === undefined && newMax !== undefined) {
        newMin = newMax - range;
    }
    range = newMax - newMin;

    const normalizedEntries = entries.map(
        (entry) => {
            const betweenZeroOne = (entry - min) / denom;
            return newMin !== undefined && newMax !== undefined ?
                newMin + range * betweenZeroOne :
                betweenZeroOne
        }
    );
    insertEntries(data, normalizedEntries);

    return normalizedEntries;
};