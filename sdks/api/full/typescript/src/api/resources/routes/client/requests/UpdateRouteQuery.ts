/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as Rivet from "../../../../index";

/**
 * @example
 *     {
 *         project: "string",
 *         environment: "string",
 *         body: {
 *             hostname: "string",
 *             path: "string",
 *             stripPrefix: true,
 *             routeSubpaths: true,
 *             target: {
 *                 actors: {
 *                     selectorTags: {}
 *                 }
 *             }
 *         }
 *     }
 */
export interface UpdateRouteQuery {
    project?: string;
    environment?: string;
    body: Rivet.routes.UpdateRouteBody;
}
