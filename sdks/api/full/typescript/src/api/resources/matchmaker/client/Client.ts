/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as environments from "../../../../environments";
import * as core from "../../../../core";
import { Lobbies } from "../resources/lobbies/client/Client";
import { Players } from "../resources/players/client/Client";
import { Regions } from "../resources/regions/client/Client";

export declare namespace Matchmaker {
    interface Options {
        environment?: core.Supplier<environments.RivetEnvironment | string>;
        token?: core.Supplier<core.BearerToken | undefined>;
        /** Override the X-API-Version header */
        xApiVersion?: "25.2.1";
        fetcher?: core.FetchFunction;
    }

    interface RequestOptions {
        /** The maximum time to wait for a response in seconds. */
        timeoutInSeconds?: number;
        /** The number of times to retry the request. Defaults to 2. */
        maxRetries?: number;
        /** A hook to abort the request. */
        abortSignal?: AbortSignal;
        /** Additional headers to include in the request. */
        headers?: Record<string, string>;
        /** Override the X-API-Version header */
        xApiVersion?: "25.2.1";
    }
}

export class Matchmaker {
    constructor(protected readonly _options: Matchmaker.Options = {}) {}

    protected _lobbies: Lobbies | undefined;

    public get lobbies(): Lobbies {
        return (this._lobbies ??= new Lobbies(this._options));
    }

    protected _players: Players | undefined;

    public get players(): Players {
        return (this._players ??= new Players(this._options));
    }

    protected _regions: Regions | undefined;

    public get regions(): Regions {
        return (this._regions ??= new Regions(this._options));
    }
}
