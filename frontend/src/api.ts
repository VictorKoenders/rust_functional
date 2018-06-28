declare namespace endpoints {
    export interface Type {
        Object?: string;
        String?: string;
    }

    export interface Input {
        name: string;
        description: string;
        type: Type;
    }

    export interface Output {
        name: string;
        description: string;
        type: Type;
    }

    export interface Method {
        name: string;
        description: string;
        input: Input[];
        output: Output[];
    }

    export interface Config2 {
        name: string;
        description: string;
        methods: Method[];
    }

    export interface Config {
        id: string;
        name: string;
        path: string;
        config: Config2;
    }

    export interface Argument {
        name: string;
        arg_type: string;
        arg_type_value: string;
    }

    export type AnyInstruction = CallMethod | JsonReturn;

    export interface CallMethod {
        config: string;
        method: string;
        out_variable_name: string;
        arguments: Argument[];
    }

    export interface JsonReturn {
        arg_type: string;
        arg_type_value: string;
    }

    export interface Instruction {
        CallMethod?: CallMethod;
        JsonReturn?: JsonReturn;
        [key: string]: AnyInstruction;
    }

    export interface Endpoint {
        id: string;
        name: string;
        description: string;
        url: string;
        instructions: Instruction[];
    }

    export interface RootObject {
        configs: Config[];
        endpoints: Endpoint[];
    }
}
