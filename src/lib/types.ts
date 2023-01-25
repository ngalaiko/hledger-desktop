export type Commodity = {
    format: string;
    name: string;
};

export namespace Commodity {
    export const fromJSON = (value: any) => ({
        format: value.format,
        name: value.name,
    });
}

export type Amount = {
    currency: string;
    value: number;
};

export namespace Amount {
    export const fromJSON = (value: any) => ({
        currency: value.currency,
        value: parseFloat(value.value),
    });
}

export type Price = {
    amount: Amount;
    commodity: string;
    date: Date;
};

export namespace Price {
    export const fromJSON = (value: any): Price => ({
        amount: Amount.fromJSON(value.amount),
        commodity: value.commodity,
        date: new Date(value.date),
    });
}

export type Journal = {
    includes: string[];
    commodities: Commodity[];
    accounts: string[];
    prices: Price[];
};

export namespace Journal {
    export const fromJSON = (value: any): Journal => ({
        includes: value?.includes ?? [],
        commodities: (value?.commodities ?? []).map(Commodity.fromJSON),
        accounts: value?.accounts ?? [],
        prices: (value?.prices ?? []).map(Price.fromJSON),
    });
}
