/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as Rivet from "../../../../../index";

export interface GetServerLogsResponse {
    /** Sorted old to new. */
    lines: string[];
    /** Sorted old to new. */
    timestamps: string[];
    watch: Rivet.WatchResponse;
}
