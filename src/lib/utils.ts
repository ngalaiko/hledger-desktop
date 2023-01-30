// TODO: this is very slow
export const mostCommon = <T extends any>(array: T[]) => {
    if (array.length === 0) return undefined;
    const translation = array.reduce((acc, val) => {
        acc.set(JSON.stringify(val), val);
        return acc;
    }, new Map<string, T>());

    const hashmap = array
        .map((v) => JSON.stringify(v))
        .reduce((acc, val) => {
            acc.set(val, (acc.get(val) || 0) + 1);
            return acc;
        }, new Map<string, number>());

    return translation.get(
        Array.from(hashmap.keys()).reduce((a, b) =>
            hashmap.get(a)! > hashmap.get(b)! ? a : b
        )
    );
};

export const average = (array: number[]) => {
    if (array.length === 0) return undefined;
    const sum = array.reduce((a, b) => a + b);
    return sum / array.length;
};
