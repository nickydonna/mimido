
import adt, { match, def, type Variants } from "@korkje/adt";
import type { EventType } from "../bindings";
import type { ParsedEvent } from "$lib/util";

export const EventUpsert = adt({
  None: null,
  Creating: (type?: EventType, startDate?: Date) => ({ type, startDate }),
  Updating: (event: ParsedEvent) => ({ event }),
});

export const eventUpserter = $state<{ state: Variants<typeof EventUpsert> }>({ state: EventUpsert.None });

export type EventUpsertVariants = Variants<typeof EventUpsert>;
export type UpdatingVariant = ReturnType<typeof EventUpsert.Updating>;
export type CreatingVariant = ReturnType<typeof EventUpsert.Creating>;
export type NoneVariant = typeof EventUpsert.None;

export function isUpdating(upserter: EventUpsertVariants): upserter is UpdatingVariant {
  return match(upserter, {
    Updating: () => true,
    [def]: () => false,
  })
}

export function isCreating(upserter: EventUpsertVariants): upserter is CreatingVariant {
  return match(upserter, {
    Creating: () => true,
    [def]: () => false,
  })
}

export function isNone(upserter: EventUpsertVariants): upserter is NoneVariant {
  return !isUpdating(upserter) && !isCreating(upserter)
}
