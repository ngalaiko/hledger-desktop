export const mostCommon = <T extends any>(array: T[]) => {
    if (array.length === 0) return undefined;
    const hashmap = array.reduce((acc, val) => {
        acc.set(val, (acc.get(val) || 0) + 1);
        return acc;
    }, new Map<T, number>());
    return Array.from(hashmap.keys()).reduce((a, b) =>
        hashmap.get(a)! > hashmap.get(b)! ? a : b
    );
};

export const average = (array: number[]) => {
    if (array.length === 0) return undefined;
    const sum = array.reduce((a, b) => a + b);
    return sum / array.length;
};
