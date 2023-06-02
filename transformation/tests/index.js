import assert from "assert";

import { transform } from "../build/debug.js";
assert.strictEqual(transform('{"name":"Adam", "age": 18}'), '{"tag":"@Adam"}');
console.log("ok");
