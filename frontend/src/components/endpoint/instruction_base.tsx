import * as React from "react";

export interface InstructionBase {
    new (...args: any[]): React.Component<{
        instruction: endpoints.AnyInstruction;
        configs: endpoints.Config[];
        stack: Stack;
        onDelete: (m: React.MouseEvent<HTMLElement>) => void;
        onChange: (m: endpoints.AnyInstruction) => void;
    }>;
    description(): string;
    create(): endpoints.Instruction;
}

export class Stack {
    constructor() {
        this.variables = {};
    }
    variables: { [key: string]: endpoints.Type };
    set_variable(name: string, type: endpoints.Type) {
        this.variables[name] = type;
    }
    clone() {
        let stack = new Stack();
        for (const key of Object.getOwnPropertyNames(this.variables)) {
            stack.set_variable(key, this.variables[key]);
        }
        return stack;
    }
}
