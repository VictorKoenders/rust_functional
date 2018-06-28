import * as React from "react";
import { Overview } from "./endpoint/overview";

interface RootProps {}

interface RootState {
    endpoints: endpoints.Endpoint[];
    configs: endpoints.Config[];
    active: endpoints.Endpoint | null;
}

export class Root extends React.Component<RootProps, RootState> {
    constructor(props: RootProps, context?: any) {
        super(props, context);
        this.state = {
            endpoints: [],
            configs: [],
            active: null
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
                    active
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

    selectEndpoint(
        endpoint: endpoints.Endpoint,
        e: React.MouseEvent<HTMLAnchorElement>
    ) {
        this.setState({
            active: endpoint
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
                    />
                ) : null}
            </div>
        );
    }
}
