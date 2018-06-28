import * as React from "react";
import { Stack } from "./instruction_base";

export type ArgType = { arg_type: string; arg_type_value: string };

export interface ArgEditorProps {
    prop: ArgType;
    propChanged: (prop: ArgType) => void;
    expected?: endpoints.Type;
    stack: Stack;
}

export interface ArgEditorState {}

export class ArgEditor extends React.Component<ArgEditorProps, ArgEditorState> {
    setValue(
        e:
            | React.ChangeEvent<HTMLSelectElement>
            | React.ChangeEvent<HTMLInputElement>
    ) {
        let prop = Object.assign({}, this.props.prop);
        prop.arg_type_value = e.target.value;
        this.props.propChanged(prop);
    }
    static new_arg() {
        return {
            arg_type: "Parameter",
            arg_type_value: ""
        };
    }
    setPropType(e: React.ChangeEvent<HTMLSelectElement>) {
        let prop = Object.assign({}, this.props.prop);
        prop.arg_type = e.target.value;
        if (prop.arg_type == "Suggested") {
            prop.arg_type = "Parameter";
            let suggested = ArgEditor.getSuggested(this.props.expected, this.props.stack);
            if(suggested.length) {
                prop.arg_type_value = suggested[0];
            }
        }
        this.props.propChanged(prop);
    }
    renderEditor(type: string, suggested: string[]) {
        switch (type) {
            case "":
                return null;
            case "Suggested":
                return (
                    <select
                        value={this.props.prop.arg_type_value}
                        onChange={this.setValue.bind(this)}
                    >
                        <option key="" value="" />
                        {suggested.map(s => (
                            <option key={s} value={s}>
                                {s}
                            </option>
                        ))}
                    </select>
                );
            case "Parameter":
                return (
                    <select
                        value={this.props.prop.arg_type_value}
                        onChange={this.setValue.bind(this)}
                    >
                        <option key="" value="" />
                        {Object.keys(this.props.stack.variables).map(
                            (v: string) => (
                                <option key={v} value={v}>
                                    {v} -{" "}
                                    {getTypeName(this.props.stack.variables[v])}
                                </option>
                            )
                        )}
                    </select>
                );
            case "String":
                return (
                    <input
                        type="text"
                        value={this.props.prop.arg_type_value}
                        onChange={this.setValue}
                    />
                );
            default:
                return (
                    <b>Unknown arg type {JSON.stringify(this.props.prop)}</b>
                );
        }
    }
    static getSuggested(expected: endpoints.Type | null, stack: Stack): string[] {
        if (!expected || !expected.Object) return [];
        let suggested = [];
        for (const v in stack.variables) {
            if (
                !stack.variables[v].String &&
                stack.variables[v].Object ==
                    expected.Object
            ) {
                suggested.push(v);
            }
        }
        return suggested;
    }

    getArgType(suggested: string[]) {
        if (
            this.props.prop.arg_type == "Parameter" &&
            suggested.some(s => s == this.props.prop.arg_type_value)
        ) {
            return "Suggested";
        }
        return this.props.prop.arg_type;
    }
    render() {
        let text = null;
        if (this.props.expected) {
            text = (
                <>
                    <br />
                    Expected {getTypeName(this.props.expected)}
                </>
            );
        }
        let suggested = ArgEditor.getSuggested(this.props.expected, this.props.stack);
        let type = this.getArgType(suggested);
        return (
            <>
                <select value={type} onChange={this.setPropType.bind(this)}>
                    <option value="" />
                    {suggested.length ? (
                        <option value="Suggested">Suggested</option>
                    ) : null}
                    <option value="Parameter">Parameter</option>
                    <option value="String">String</option>
                </select>
                {this.renderEditor(type, suggested)}
                {text}
            </>
        );
    }
}

export function getTypeName(type: endpoints.Type) {
    if (type.Object) {
        return "object " + type.Object;
    }
    if (type.String) {
        return "text";
    }
    return "Unknown type";
}
