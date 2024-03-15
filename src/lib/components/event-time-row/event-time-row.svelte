<script>
  import Event from "../event/event.svelte";
	import { addMinutes, eachHourOfInterval, format, getHours, isWithinInterval, setHours, setMinutes, setSeconds, subSeconds } from "date-fns/fp";

  /** @type {Array<import("$lib/server/schemas/event").TEventSchema>} */
  export let events
  /** @type {Date} */
  export let time
  /** @type {Date} */
  export let displayDate



	/**
	 * @param {number} startHour
	 * @param {number} minOffset
	 * @param {import("../task-box/parser").TEventSchema} event
	 * @returns {boolean}
	 */
	let timeCheck = (startHour, minOffset, event) => {
		if (!event.date || !event.hasStartTime) {
			return false;
		}
  
		const low = subSeconds(1, setMinutes(minOffset, setHours(startHour, displayDate)));
		const high = subSeconds(1, setMinutes(minOffset + 30, setHours(startHour, displayDate)))
		return isWithinInterval({
			start: low,
			end: high
		}, event.date);
	}
</script>
<tr class="h-8 border-b p-0">
	<td>{format('HH:mm', time)}</td>
	<td class="p-0">
		{#each events.filter((e) => timeCheck(getHours(time), 0, e)) as e}
			<Event event={e} />
		{/each}
	</td>
</tr>
<tr class="h-8 border-b border-gray-400">
	<td class="text-white">{format('HH:mm', addMinutes(30, time))}</td>
	<td class="p-0">
		{#each events.filter((e) => timeCheck(getHours(time), 30, e)) as e}
			<Event event={e} />
		{/each}
	</td>
</tr>
