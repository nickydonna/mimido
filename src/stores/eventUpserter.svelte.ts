
import adt, { type Variants } from "@korkje/adt";
import type { EventType } from "../bindings";
import type { ParsedEvent } from "$lib/util";

export const eventUpsert = adt({
  None: null,
  Creating: (type: EventType, startDate: Date) => ({ type, startDate }),
  Updating: (event: ParsedEvent) => ({ event }),
});

export const eventUpserter = $state<{ state: Variants<typeof eventUpsert> }>({ state: eventUpsert.None });
