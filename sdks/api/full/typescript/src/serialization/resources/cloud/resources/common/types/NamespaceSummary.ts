/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { Timestamp } from "../../../../common/types/Timestamp";
import { DisplayName } from "../../../../common/types/DisplayName";

export const NamespaceSummary: core.serialization.ObjectSchema<
    serializers.cloud.NamespaceSummary.Raw,
    Rivet.cloud.NamespaceSummary
> = core.serialization.object({
    namespaceId: core.serialization.property("namespace_id", core.serialization.string()),
    createTs: core.serialization.property("create_ts", Timestamp),
    displayName: core.serialization.property("display_name", DisplayName),
    versionId: core.serialization.property("version_id", core.serialization.string()),
    nameId: core.serialization.property("name_id", core.serialization.string()),
});

export declare namespace NamespaceSummary {
    export interface Raw {
        namespace_id: string;
        create_ts: Timestamp.Raw;
        display_name: DisplayName.Raw;
        version_id: string;
        name_id: string;
    }
}
