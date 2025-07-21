import { getHours, getMinutes, setHours, setMinutes } from "date-fns";

export const timeState = $state<{ time: Date, nextSlot: Date }>({ time: new Date(), nextSlot: new Date() })

setInterval(() => {
  const now = new Date()
  timeState.time = now;
  const minutes = getMinutes(now);
  if (minutes > 1) {
    timeState.nextSlot = setMinutes(now, 30)
  } else if (minutes > 31) {
    timeState.nextSlot = setHours(setMinutes(now, 0), getHours(now))
  } else {
    timeState.nextSlot = setMinutes(now, 0);
  }
}, 10000);



