export const mostCommon = <T extends string>(array: T[]) => {
    if (array.length === 0) return undefined;
    const hashmap = array.reduce((acc, val) => {
        acc[val] = (acc[val] || 0) + 1;
        return acc;
    }, {} as Record<T, number>);
    return (Object.keys(hashmap) as T[]).reduce((a, b) =>
        hashmap[a] > hashmap[b] ? a : b
    );
};

export const average = (array: number[]) => {
    if (array.length === 0) return undefined;
    const sum = array.reduce((a, b) => a + b);
    return sum / array.length;
};
