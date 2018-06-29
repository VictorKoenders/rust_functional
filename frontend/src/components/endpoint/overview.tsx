import * as React from "react";
import cloneDeep = require("lodash/cloneDeep");
import { CallMethod } from "./call_method";
import { JsonReturn } from "./json_return";
import { InstructionBase, Stack } from "./instruction_base";

const instruction_renderers: { [key: string]: InstructionBase } = {
    CallMethod: CallMethod,
    JsonReturn: JsonReturn
};

export class OverviewProps {
    endpoint: endpoints.Endpoint;
    endpointChanged: (e: endpoints.Endpoint) => void;
    configs: endpoints.Config[];
    changeIndex: number;
}

export class OverviewState {
    endpoint: endpoints.Endpoint;
    changeIndex: number;
    hasChanges: boolean;
    output: string | null;
}

export class Overview extends React.Component<OverviewProps, OverviewState> {
    drag_from: React.RefObject<HTMLUListElement>;
    drag_to: React.RefObject<HTMLUListElement>;
    dragger: Dragger | null;
    constructor(props: OverviewProps, context?: any) {
        super(props, context);
        this.state = {
            endpoint: cloneDeep(props.endpoint),
            hasChanges: false,
            changeIndex: props.changeIndex,
            output: null
        };
        this.drag_from = React.createRef();
        this.drag_to = React.createRef();
        this.dragger = null;
    }
    static getDerivedStateFromProps(
        props: OverviewProps,
        oldState: OverviewState
    ) {
        if (props.changeIndex != oldState.changeIndex) {
            return {
                endpoint: cloneDeep(props.endpoint),
                changeIndex: props.changeIndex,
                hasChanges: false,
            };
        } else {
            return null;
        }
    }

    deleteInstruction(index: number, ev: React.MouseEvent<HTMLElement>) {
        ev.preventDefault();
        ev.stopPropagation();

        let endpoint = Object.assign({}, this.state.endpoint);
        endpoint.instructions.splice(index, 1);

        this.setState({ endpoint, hasChanges: true });

        return false;
    }

    changeInstruction(index: number, key: string, instruction: any) {
        this.setState(state => {
            let old_output_name =
                this.state.endpoint.instructions[index] &&
                this.state.endpoint.instructions[index].CallMethod
                    ? this.state.endpoint.instructions[index].CallMethod
                          .out_variable_name
                    : null;
            let endpoint = Object.assign({}, state.endpoint);
            endpoint.instructions[index] = {};
            endpoint.instructions[index][key] = instruction;
            if (
                old_output_name != null &&
                old_output_name != instruction.out_variable_name
            ) {
                let new_name = instruction.out_variable_name;
                for (var i = index + 1; i < endpoint.instructions.length; i++) {
                    let instruction = endpoint.instructions[i];
                    if (instruction.CallMethod) {
                        for (const input of instruction.CallMethod.arguments) {
                            if (
                                input.arg_type == "Parameter" &&
                                input.arg_type_value == old_output_name
                            ) {
                                input.arg_type_value = new_name;
                            }
                        }
                        if (
                            instruction.CallMethod.out_variable_name ==
                            old_output_name
                        ) {
                            break;
                        }
                    }
                }
            }
            return {
                endpoint,
                hasChanges: true
            };
        });
    }

    componentInserted(original: HTMLLIElement, index: number) {
        let endpoint = Object.assign({}, this.state.endpoint);
        let instruction_renderer =
            instruction_renderers[original.dataset.instruction];
        let instruction = instruction_renderer.create();
        endpoint.instructions.splice(index, 0, instruction);
        this.setState({
            endpoint,
            hasChanges: true
        });
    }

    componentDidMount() {
        this.dragger = new Dragger(
            this.drag_from.current,
            this.drag_to.current,
            this.componentInserted.bind(this)
        );
    }
    save() {
        this.props.endpointChanged(this.state.endpoint);
    }
    clearOutput() {
        this.setState({
            output: null
        });
    }
    generate() {
        fetch("/api/generate/" + this.state.endpoint.id)
            .then(o => o.text())
            .then(t => {
                this.setState({
                    output: t
                });
            });
    }
    render() {
        let stack = new Stack();
        if (this.state.output !== null) {
            return (
                <>
                    <button
                        className="btn float-right btn-primary"
                        onClick={this.clearOutput.bind(this)}
                    >
                        &times;
                    </button>
                    <pre>
                        <code>{this.state.output}</code>
                    </pre>
                </>
            );
        }
        return (
            <div className="row">
                <div className="col-md-8">
                    <div className="row">
                        <div className="col-md-12">
                            <button
                                className="btn float-right btn-primary"
                                onClick={(this.state.hasChanges
                                    ? this.save
                                    : this.generate
                                ).bind(this)}
                            >
                                {this.state.hasChanges ? "Save" : "Generate"}
                            </button>
                        </div>
                    </div>
                    <ul ref={this.drag_to}>
                        {this.state.endpoint.instructions.map(
                            (i: endpoints.Instruction, index: number) => {
                                let key = Object.getOwnPropertyNames(i)[0];
                                const Renderer = instruction_renderers[key];
                                return (
                                    <Renderer
                                        key={index}
                                        instruction={i[key]}
                                        configs={this.props.configs}
                                        stack={stack}
                                        onDelete={this.deleteInstruction.bind(
                                            this,
                                            index
                                        )}
                                        onChange={this.changeInstruction.bind(
                                            this,
                                            index,
                                            key
                                        )}
                                    />
                                );
                            }
                        )}
                    </ul>
                </div>
                <div className="col-md-4">
                    <h4>Drag and drop to add:</h4>
                    <ul ref={this.drag_from}>
                        {Object.keys(instruction_renderers).map(key => (
                            <li key={key} data-instruction={key}>
                                {instruction_renderers[key].description()}
                            </li>
                        ))}
                    </ul>
                </div>
            </div>
        );
    }
}

class Dragger {
    from: HTMLUListElement;
    to: HTMLUListElement;
    callback: (original: HTMLLIElement, index: number) => void;
    original_drag_element: HTMLLIElement | null;
    drag_element: HTMLDivElement | null;
    hover: HTMLLIElement | null;
    constructor(
        from: HTMLUListElement,
        to: HTMLUListElement,
        callback: (original: HTMLLIElement, index: number) => void
    ) {
        this.from = from;
        this.to = to;
        this.callback = callback;
        this.drag_element = null;
        this.hover = null;

        this.from.addEventListener("mousedown", this.from_down.bind(this));
        document.body.addEventListener("mouseup", this.clearDrag.bind(this));
        document.body.addEventListener("mousemove", this.move.bind(this));
    }

    from_down(ev: MouseEvent) {
        ev.preventDefault();
        ev.stopPropagation();
        let target = ev.target as HTMLLIElement;
        let drag_element = document.createElement("div");
        for (let child of target.children) {
            drag_element.appendChild(child.cloneNode(true));
        }
        drag_element.style.position = "fixed";
        drag_element.style.backgroundColor = "white";
        drag_element.style.border = "1px solid black";

        let box = target.getBoundingClientRect();
        drag_element.style.top = box.top + "px";
        drag_element.style.left = box.left + "px";
        drag_element.style.width = box.width + "px";
        drag_element.style.height = box.height + "px";

        drag_element.dataset.offset_x = (ev.clientX - box.left).toFixed(0);
        drag_element.dataset.offset_y = (ev.clientY - box.top).toFixed(0);

        for (let child of target.childNodes) {
            drag_element.appendChild(child.cloneNode(true));
        }

        document.body.appendChild(drag_element);
        this.original_drag_element = target;
        this.drag_element = drag_element;
        return false;
    }

    move(ev: MouseEvent) {
        if (!this.drag_element) return true;
        let offset_x = parseInt(this.drag_element.dataset.offset_x);
        let offset_y = parseInt(this.drag_element.dataset.offset_y);

        this.drag_element.style.left = ev.pageX - offset_x + "px";
        this.drag_element.style.top = ev.pageY - offset_y + "px";

        let dragging_box = this.drag_element.getBoundingClientRect();
        let dragging_box_middle = dragging_box.top + dragging_box.height / 2;

        if (
            !rectContainsHorizontal(
                this.to.getBoundingClientRect(),
                dragging_box
            )
        ) {
            return true;
        }

        let before = null;
        for (let i = 0; i < this.to.children.length; i++) {
            let child = this.to.children[i];
            let box = child.getBoundingClientRect();
            let middle = box.top + box.height / 2;
            if (child.classList.contains("drag-hover")) {
                if (rectContains(box, dragging_box)) {
                    // We're hovering over the current item, don't do anything
                    return;
                }
                continue;
            }
            if (middle > dragging_box_middle) {
                before = child;
                break;
            }
        }

        if (this.hover != null) {
            this.hover.parentNode.removeChild(this.hover);
            this.hover = null;
        }
        let hover = document.createElement("li");
        hover.classList.add("drag-hover");
        if (before) {
            this.to.insertBefore(hover, before);
        } else {
            this.to.appendChild(hover);
        }
        this.hover = hover;
        return true;
    }

    clearDrag() {
        let elem =
            this.hover != null && !this.drag_element != null
                ? {
                      index: Array.prototype.indexOf.call(
                          this.hover.parentNode.childNodes,
                          this.hover
                      ),
                      elem: this.original_drag_element
                  }
                : null;
        if (this.hover != null) {
            this.hover.parentNode.removeChild(this.hover);
            this.hover = null;
        }
        if (this.drag_element != null) {
            this.drag_element.parentElement.removeChild(this.drag_element);
            this.drag_element = null;
            this.original_drag_element = null;
        }
        if (elem != null) {
            this.callback(elem.elem, elem.index);
        }
    }
}

function rectContains(rect1: ClientRect, rect2: ClientRect) {
    return !(
        rect1.right < rect2.left ||
        rect1.left > rect2.right ||
        rect1.bottom < rect2.top ||
        rect1.top > rect2.bottom
    );
}
function rectContainsHorizontal(rect1: ClientRect, rect2: ClientRect) {
    return !(rect1.right < rect2.left || rect1.left > rect2.right);
}
