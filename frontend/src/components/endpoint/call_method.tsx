import * as React from "react";
import { Stack, guid } from "./instruction_base";
import { ArgEditor, getTypeName, argToString } from "./arg_editor";

interface CallMethodProps {
    instruction: endpoints.CallMethod;
    configs: endpoints.Config[];
    stack: Stack;
    onDelete: (ev: React.MouseEvent<HTMLElement>) => void;
    onChange: (m: endpoints.CallMethod) => void;
}

interface CallMethodState {
    collapsed: boolean;
}

export class CallMethod extends React.Component<
    CallMethodProps,
    CallMethodState
> {
    static description() {
        return "Call a method from a config";
    }
    static create(): endpoints.Instruction {
        return {
            CallMethod: {
                id: guid(),
                config: "",
                method: "",
                out_variable_name: "",
                arguments: [] as endpoints.Argument[]
            }
        };
    }
    constructor(props: CallMethodProps, context?: any) {
        super(props, context);
        this.state = {
            collapsed: true
        };
    }
    setConfig(ev: React.ChangeEvent<HTMLSelectElement>) {
        let instruction = Object.assign({}, this.props.instruction);
        instruction.config = ev.target.value;
        this.props.onChange(instruction);
        this.setState({
            collapsed: false,
        })
    }
    setMethod(ev: React.ChangeEvent<HTMLSelectElement>) {
        let instruction = Object.assign({}, this.props.instruction);
        instruction.method = ev.target.value;
        instruction.arguments = [];
        let config = this.props.configs.find(c => c.id == instruction.config);
        if (config) {
            let method = config.config.methods.find(
                m => m.name == instruction.method
            );
            if (method) {
                for (const input of method.input) {
                    let suggested = ArgEditor.getSuggested(
                        input.type,
                        this.props.stack
                    );
                    instruction.arguments.push({
                        name: input.name,
                        arg_type: input.type.Object
                            ? "Parameter"
                            : input.type.String
                                ? "String"
                                : "",
                        arg_type_value: suggested.length ? suggested[0] : ""
                    });
                }
                if (method.output.length) {
                    let name = method.output[0].name;
                    if (this.props.stack.variables[name]) {
                        for (let n = 0; ; n++) {
                            if (!this.props.stack.variables[name + n]) {
                                name += n.toString();
                                break;
                            }
                        }
                    }
                    instruction.out_variable_name = name;
                } else {
                    instruction.out_variable_name = "";
                }
            }
        }

        this.props.onChange(instruction);
    }
    setOutVariableName(ev: React.ChangeEvent<HTMLInputElement>) {
        let instruction = Object.assign({}, this.props.instruction);
        instruction.out_variable_name = ev.target.value;
        this.props.onChange(instruction);
    }
    setArgumentName(index: number, ev: React.ChangeEvent<HTMLSelectElement>) {
        let instruction = Object.assign({}, this.props.instruction);
        instruction.arguments[index].name = ev.target.value;
        this.props.onChange(instruction);
    }
    propChanged(
        arg: endpoints.Input,
        prop: { arg_type: string; arg_type_value: string }
    ) {
        let instruction = Object.assign({}, this.props.instruction);
        let index = instruction.arguments.findIndex(a => a.name == arg.name);
        if (index == -1) {
            instruction.arguments.push(
                Object.assign(
                    {
                        name: arg.name
                    },
                    prop
                )
            );
        } else {
            instruction.arguments[index].arg_type = prop.arg_type;
            instruction.arguments[index].arg_type_value = prop.arg_type_value;
        }
        this.props.onChange(instruction);
    }

    renderArgument(arg: endpoints.Input, index: number) {
        let prop = this.props.instruction.arguments.find(
            a => a.name == arg.name
        ) || {
            name: arg.name,
            arg_type: "String",
            arg_type_value: ""
        };
        return (
            <tr key={arg.name}>
                <td>
                    <b>{arg.name}</b>
                </td>
                <td>
                    <ArgEditor
                        prop={prop}
                        stack={this.props.stack}
                        expected={arg.type}
                        propChanged={this.propChanged.bind(this, arg)}
                    />
                </td>
            </tr>
        );
    }
    toggleCollapse(){
        this.setState({
            collapsed: !this.state.collapsed,
        })
    }
    render() {
        let config = this.props.configs.find(
            c => c.id == this.props.instruction.config
        );
        let config_select = (
            <select
                onChange={this.setConfig.bind(this)}
                value={this.props.instruction.config}
            >
                <option value="" />
                {this.props.configs.map(c => (
                    <option value={c.id} key={c.id}>
                        {c.name}
                    </option>
                ))}
            </select>
        );
        if (!config) {
            return <li>{config_select}</li>;
        }
        let method = config.config.methods.find(
            m => m.name == this.props.instruction.method
        );
        let method_select = (
            <select
                onChange={this.setMethod.bind(this)}
                value={this.props.instruction.method}
            >
                <option value="" />
                {config.config.methods.map(m => (
                    <option value={m.name} key={m.name}>
                        {m.name}
                    </option>
                ))}
            </select>
        );
        if (!method) {
            return (
                <li>
                    {config_select}
                    {method_select}
                </li>
            );
        }
        if (method.output.length > 0) {
            this.props.stack.set_variable(
                this.props.instruction.out_variable_name,
                method.output[0].type
            );
        }
        if (this.state.collapsed) {
            return (
                <li
                    style={{
                        padding: "5px",
                        minHeight: "40px",
                    }}
                >
                    <button className="btn btn-success float-right" onClick={this.toggleCollapse.bind(this)}>
                        Edit
                    </button>
                    <code>
                        {method.output.length > 0 ? (
                            <>
                                let{" "}
                                <b>
                                    {this.props.instruction.out_variable_name}
                                </b>{" "}
                                ={" "}
                            </>
                        ) : null}
                        <b>{config.name}</b>::<b>{method.name}</b>(
                        {this.props.instruction.arguments.map(a => (
                            <p key={a.name} style={{marginBottom: 0, marginLeft: "1rem"}}>{argToString(a)}, </p>
                        ))}
                        );<br />
                    </code>
                </li>
            );
        }
        let output = null;
        if (method.output.length > 0) {
            output = (
                <>
                    returning <b>{method.output[0].name}</b> ({getTypeName(
                        method.output[0].type
                    )}) as:{" "}
                    <input
                        type="text"
                        value={this.props.instruction.out_variable_name}
                        onChange={this.setOutVariableName.bind(this)}
                    />
                </>
            );
        }
        return (
            <li>
                <div className="float-right">
                    <button className="btn btn-success" onClick={this.toggleCollapse.bind(this)}>
                        Collapse
                    </button>
                    <button
                        className="btn btn-danger"
                        onClick={this.props.onDelete}
                    >
                        &times;
                    </button>
                </div>
                {config_select}
                {method_select}
                <table>
                    <tbody>
                        {method.input.map(this.renderArgument.bind(this))}
                    </tbody>
                </table>
                {output}
            </li>
        );
    }
}
