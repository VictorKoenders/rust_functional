import * as React from "react";
import { Overview } from "./endpoint/overview";

interface RootProps {}

interface RootState {
    endpoints: endpoints.Endpoint[];
    configs: endpoints.Config[];
    active: endpoints.Endpoint | null;
    activeIndex: number,
}

export class Root extends React.Component<RootProps, RootState> {
    constructor(props: RootProps, context?: any) {
        super(props, context);
        this.state = {
            endpoints: [],
            configs: [],
            active: null,
            activeIndex: 0,
        };

        fetch("/api/endpoints")
            .then(r => r.json())
            .then((r: endpoints.RootObject) => {
                let active =
                    r.endpoints.find(
                        (e: endpoints.Endpoint) =>
                            "#" + e.name == document.location.hash
                    ) ||
                    r.endpoints[0] ||
                    null;
                this.setState({
                    endpoints: r.endpoints,
                    configs: r.configs,
                    active,
                    activeIndex: 1,
                });
            });
    }

    renderEndpoint(endpoint: endpoints.Endpoint, index: number) {
        let className = "nav-link";
        if (endpoint == this.state.active) {
            className += " active";
        }
        return (
            <li className="nav-item" key={endpoint.id}>
                <a
                    href={"/#" + endpoint.name}
                    className={className}
                    onClick={this.selectEndpoint.bind(this, endpoint)}
                >
                    {endpoint.name}
                </a>
            </li>
        );
    }

    endpointChanged(endpoint: endpoints.Endpoint) {
        let index = this.state.endpoints.findIndex(e => e.id == endpoint.id);
        let endpoints = Object.assign([], this.state.endpoints);
        endpoints[index] = endpoint;
        this.setState({
            endpoints
        });

        fetch("/api/endpoints", {
            body: JSON.stringify(endpoint),
            headers: {
                "content-type": "application/json"
            },
            method: "POST"
        })
            .then(r => r.json())
            .then(r => {
                let index = this.state.endpoints.findIndex(e => e.id == r.id);
                let endpoints = Object.assign([], this.state.endpoints);
                endpoints[index] = r;
                this.setState({
                    endpoints,
                    active: r,
                    activeIndex: this.state.activeIndex + 1,
                });
            });
    }

    selectEndpoint(
        endpoint: endpoints.Endpoint,
        e: React.MouseEvent<HTMLAnchorElement>
    ) {
        this.setState({
            active: endpoint,
            activeIndex: this.state.activeIndex + 1,
        });
        e.currentTarget.blur();
    }

    render() {
        return (
            <div>
                <ul className="nav nav-tabs">
                    {this.state.endpoints.map(this.renderEndpoint.bind(this))}
                </ul>
                {this.state.active ? (
                    <Overview
                        endpoint={this.state.active}
                        configs={this.state.configs}
                        changeIndex={this.state.activeIndex}
                        endpointChanged={this.endpointChanged.bind(this)}
                    />
                ) : null}
            </div>
        );
    }
}
