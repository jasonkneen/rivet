/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../../../index";
import * as Rivet from "../../../../../../../../api/index";
import * as core from "../../../../../../../../core";

export const DeleteMatchmakerLobbyResponse: core.serialization.ObjectSchema<
    serializers.cloud.games.DeleteMatchmakerLobbyResponse.Raw,
    Rivet.cloud.games.DeleteMatchmakerLobbyResponse
> = core.serialization.object({
    didRemove: core.serialization.property("did_remove", core.serialization.boolean()),
});

export declare namespace DeleteMatchmakerLobbyResponse {
    export interface Raw {
        did_remove: boolean;
    }
}
