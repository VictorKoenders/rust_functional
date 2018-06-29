import * as React from "react";
import { Stack, guid } from "./instruction_base";
import { ArgEditor } from "./arg_editor";

interface JsonReturnProps {
    instruction: endpoints.JsonReturn;
    configs: endpoints.Config[];
    stack: Stack;
    onDelete: (ev: React.MouseEvent<HTMLElement>) => void;
    onChange: (m: endpoints.JsonReturn) => void;
}

interface JsonReturnState {}

export class JsonReturn extends React.Component<
    JsonReturnProps,
    JsonReturnState
> {
    static description() {
        return "return a JSON value";
    }
    static create(): endpoints.Instruction {
        return {
            JsonReturn: Object.assign({ id: guid() }, ArgEditor.new_arg())
        };
    }
    propChanged(prop: { arg_type: string; arg_type_value: string }) {
        this.props.onChange(
            Object.assign({ id: this.props.instruction.id }, prop)
        );
    }
    render() {
        return (
            <li>
                <a
                    href="#"
                    className="float-right btn btn-danger"
                    onClick={this.props.onDelete}
                >
                    &times;
                </a>
                <p>Returning json of:</p>
                <ArgEditor
                    prop={this.props.instruction}
                    stack={this.props.stack}
                    propChanged={this.propChanged.bind(this)}
                />
            </li>
        );
    }
}
