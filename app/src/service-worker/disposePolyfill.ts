//@ts-expect-error Not defined in Safari but TS does not know that
Symbol.dispose ??= Symbol("Symbol.dispose");
//@ts-expect-error Not defined in Safari but TS does not know that
Symbol.asyncDispose ??= Symbol("Symbol.asyncDispose");
