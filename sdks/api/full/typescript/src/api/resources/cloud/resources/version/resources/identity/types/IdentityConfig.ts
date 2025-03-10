/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as Rivet from "../../../../../../../index";

/**
 * **Deprecated**
 * Identity configuration for a given version.
 */
export interface IdentityConfig {
    /** **Deprecated** */
    displayNames?: string[];
    /** **Deprecated** */
    avatars?: string[];
    /** **Deprecated** */
    customDisplayNames?: Rivet.cloud.version.identity.CustomDisplayName[];
    /** **Deprecated** */
    customAvatars?: Rivet.cloud.version.identity.CustomAvatar[];
}
