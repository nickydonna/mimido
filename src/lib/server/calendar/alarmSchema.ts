import yup from 'yup';
import type { InferType } from 'yup';

export const alarmSchema = yup
	.object({
		related: yup.string().matches(/START/).required(),
		isNegative: yup.boolean().required(),
		duration: yup
			.object({
				weeks: yup.number(),
				days: yup.number(),
				hours: yup.number(),
				minutes: yup.number()
			})
			.required()
	})
	.required();

export type TAlarmSchema = InferType<typeof alarmSchema>;
