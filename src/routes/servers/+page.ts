import { commands } from "../../bindings";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ url }) => {
  const servers = await commands.listServers();
  return { servers };

}
