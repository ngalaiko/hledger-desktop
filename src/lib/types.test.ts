import { test } from "uvu";
import { equal } from "uvu/assert";
import { Amount } from "./types";

test("Amount.format.small fraction", () =>
    equal(
        Amount.format({
            acommodity: "BTC",
            aprice: null,
            aquantity: {
                decimalMantissa: 2107437,
                decimalPlaces: 8,
                floatingPoint: 0.02107437,
            },
            aismultiplier: false,
            astyle: {
                ascommodityside: "R",
                ascommodityspaced: true,
                asdecimalpoint: ".",
                asdigitgroups: [",", [3]],
                asprecision: 8,
            },
        }),
        "0.02107437 BTC"
    ));

test("Amount.format.thousands separator", () =>
    equal(
        Amount.format({
            acommodity: "SEK",
            aprice: null,
            aquantity: {
                decimalMantissa: -3332500,
                decimalPlaces: 2,
                floatingPoint: -33325,
            },
            aismultiplier: false,
            astyle: {
                ascommodityside: "R",
                ascommodityspaced: true,
                asdecimalpoint: ".",
                asdigitgroups: [",", [3]],
                asprecision: 2,
            },
        }),
        "-33,325.00 SEK"
    ));

test.run();
