export type Quantity = {
    decimalMantissa: number;
    decimalPlaces: number;
    floatingPoint: number;
};

export type AmountStyle = {
    ascommodityside: "R" | "L";
    ascommodityspaced: boolean;
    asprecision: number;
    asdecimalpoint: string | null;
    asdigitgroups: [string, number[]] | null;
};

export type Price = {
    contents: Amount;
    tag: "TotalPrice" | "UnitPrice";
};

export type Amount = {
    acommodity: string;
    aprice: Price | null;
    aquantity: Quantity;
    aismultiplier: boolean;
    astyle: AmountStyle;
};

export namespace Amount {
    const formatGroups = (
        groups: [string, number[]] | null,
        amount: string
    ): string => {
        if (groups === null) return amount;
        const separator = groups[0];
        const digitGroups = groups[1];

        if (digitGroups.length === 0) return amount;
        const lastGroup = digitGroups.at(-1)!;
        if (lastGroup >= amount.length) return amount;

        return (
            formatGroups(
                [
                    separator,
                    digitGroups.length === 1 ? digitGroups : digitGroups.slice(0, -1),
                ],
                amount.slice(0, -1 * lastGroup)
            ) +
            separator +
            amount.slice(-1 * lastGroup)
        );
    };

    const formatQuantity = (style: AmountStyle, quantity: Quantity): string => {
        const isNegative = quantity.floatingPoint < 0;
        const formatted = Math.abs(quantity.floatingPoint).toFixed(
            style.asprecision
        );
        const [beforeSeparator, afterSeparator] = formatted.split(".");
        return (
            (isNegative ? "-" : "") +
            formatGroups(style.asdigitgroups, beforeSeparator) +
            (style.asdecimalpoint ?? ".") +
            afterSeparator
        );
    };

    export const format = (amount: Amount): string => {
        const value = formatQuantity(amount.astyle, amount.aquantity);
        const commodity = amount.acommodity.includes(" ")
            ? `"${amount.acommodity}"`
            : amount.acommodity;
        const space = amount.astyle.ascommodityspaced ? " " : "";

        const result =
            amount.astyle.ascommodityside === "R"
                ? `${value}${space}${commodity}`
                : `${commodity}${space}${value}`;

        if (amount.aprice === null) {
            return result;
        } else if (amount.aprice.tag === "TotalPrice") {
            return [result, "@@", format(amount.aprice.contents)].join(" ");
        } else {
            return [result, "@", format(amount.aprice.contents)].join(" ");
        }
    };
}

export type Account = {
    aname: string;
    aebalance: Amount[];
    aibalance: Amount[];
    anumpostings: number;
    aparent_: string;
    asubs_: string[];
};

export type BalanceAssertion = {
    baamount: Amount;
    bainclusive: boolean;
    baposition: SourcePos;
    batotal: boolean;
};

export type Posting = {
    paccount: string;
    pamount: [Amount];
    pbalanceassertion: BalanceAssertion | null;
    pcomment: string;
    pdate: string | null;
    pdate2: string | null;
    poriginal: Posting | null;
    pstatus: "Unmarked" | "Pending" | "Cleared";
    ptags: string[];
    ptransaction_: number;
    ptype: "RegularPosting" | "VirtualPosting" | "BalancedVirtualPosting";
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
