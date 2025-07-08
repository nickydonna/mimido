import { unwrap } from "$lib/result";
import { commands } from "../../bindings";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ url }) => {
  const result = await commands.listServers();

  return { servers: unwrap(result) };

}
