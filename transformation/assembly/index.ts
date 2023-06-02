import { JSON } from "json-as/assembly";

@json
class Player {
  name!: string;
  age!: i32;
}

@json
class Output {
  tag: string
}

// Create the transformation
export function transform(json: string): string {
    const player = JSON.parse<Player>(json);
    //log("transforming player with name " + player.name);
    const output: Output = { tag: "@"+player.name }
    const output_str: String = JSON.stringify(output)
    return output_str
}

// this should be conditional on the build target
declare function log(s: String): void;





