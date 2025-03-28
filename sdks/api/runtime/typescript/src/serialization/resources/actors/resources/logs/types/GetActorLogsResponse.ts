/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { Timestamp } from "../../../../common/types/Timestamp";
import { WatchResponse } from "../../../../common/types/WatchResponse";

export const GetActorLogsResponse: core.serialization.ObjectSchema<
    serializers.actors.GetActorLogsResponse.Raw,
    Rivet.actors.GetActorLogsResponse
> = core.serialization.object({
    lines: core.serialization.list(core.serialization.string()),
    timestamps: core.serialization.list(Timestamp),
    watch: WatchResponse,
});

export declare namespace GetActorLogsResponse {
    interface Raw {
        lines: string[];
        timestamps: Timestamp.Raw[];
        watch: WatchResponse.Raw;
    }
}
