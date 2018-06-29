import * as React from "react";
import { Stack, guid } from "./instruction_base";
import { ArgEditor, argToString } from "./arg_editor";

interface JsonReturnProps {
    instruction: endpoints.JsonReturn;
    configs: endpoints.Config[];
    stack: Stack;
    onDelete: (ev: React.MouseEvent<HTMLElement>) => void;
    onChange: (m: endpoints.JsonReturn) => void;
}

interface JsonReturnState {
    collapsed: boolean;
}

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
    constructor(props: JsonReturnProps, context?: any) {
        super(props, context);
        this.state = {
            collapsed: true
        };
    }
    propChanged(prop: { arg_type: string; arg_type_value: string }) {
        this.props.onChange(
            Object.assign({ id: this.props.instruction.id }, prop)
        );
    }
    toggleCollapse() {
        this.setState({
            collapsed: !this.state.collapsed,
        })
    }
    render() {
        if (this.state.collapsed) {
            return (
                <li
                    style={{
                        padding: "5px",
                        minHeight: "40px"
                    }}
                >
                    <button className="btn btn-success float-right" onClick={this.toggleCollapse.bind(this)}>
                        Edit
                    </button>
                    <code>
                        return Json({argToString(this.props.instruction)});
                    </code>
                </li>
            );
        }
        return (
            <li>
                <button className="btn btn-success float-right" onClick={this.toggleCollapse.bind(this)}>
                    Collapse
                </button>
                <button
                    className="float-right btn btn-danger"
                    onClick={this.props.onDelete}
                >
                    &times;
                </button>
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
