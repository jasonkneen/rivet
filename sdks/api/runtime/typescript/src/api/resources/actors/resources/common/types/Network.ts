/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as Rivet from "../../../../../index";

export interface Network {
    mode: Rivet.actors.NetworkMode;
    ports: Record<string, Rivet.actors.Port>;
}
