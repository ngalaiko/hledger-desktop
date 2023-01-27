export type Quantity = {
    decimalMantissa: number;
    decimalPlaces: number;
    floatingPoint: number;
};

export type AmountStyle = {
    ascommodityside: "R" | "L";
    ascommodityspaced: boolean;
    asdecimalpoint: string;
    asdigitgroup: [string, [number]];
    asprecision: number;
};

export type Price = {
    contents: Amount;
    tag: "TotalPrice";
};

export type Amount = {
    acommodity: string;
    aprice: Price | null;
    aquantity: Quantity;
    astyle: AmountStyle;
};

export type Account = {
    aname: string;
    aebalance: Amount[];
    aibalance: Amount[];
    anumpostings: number;
    aparent_: string;
    asubs_: string[];
};

export type Posting = {
    paccount: string;
    pamount: [Amount];
    pbalanceassertion: string;
    pcomment: string;
    pstatus: "Unmarked";
    ptags: string[];
    ptransaction_: number;
    ptype: "RegularPosting";
};

export type SourcePos = {
    sourceLine: number;
    sourceColumn: number;
    sourceName: string;
};

export type Transaction = {
    tindex: number;
    tdate: string;
    tdate2: string;
    tdescription: string;
    tpostings: Posting[];
    tprecedingcomment: string;
    tcomment: string;
    tsourcepos: [SourcePos, SourcePos];
    tstatus: "Unmarked";
    ttags: string[];
};
