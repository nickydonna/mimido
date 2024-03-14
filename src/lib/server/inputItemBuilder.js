import { AttributeValue } from '@aws-sdk/client-dynamodb';
import { formatISO } from 'date-fns/fp';

export class InputItemBuilder {
	/**
	 *
	 * @type {Record<string, AttributeValue>}
	 */
	object = {};

	static create() {
		return new InputItemBuilder();
	}

	/**
	 *
	 * @param {string} key
	 * @param {string | undefined} value
	 * @returns {InputItemBuilder}
	 */
	addS(key, value) {
		if (typeof value === 'undefined') return this;
		this.object = { ...this.object, [key]: { S: value } };
		return this;
	}

	/**
	 * @template T
	 * @param {string} key
	 * @param {T | undefined}value
	 * @param {(s: T) => string}formatter
	 * @returns {InputItemBuilder}
	 */
	addSWithFormat(key, value, formatter) {
		if (typeof value === 'undefined') return this;
		return this.addS(key, formatter(value));
	}

	/**
	 *
	 * @param {string} key
	 * @param {Date | undefined} value
	 * @returns {InputItemBuilder}
	 */
	addSDate(key, value) {
		return this.addSWithFormat(key, value, formatISO);
	}

	/**
	 *
	 * @returns {Record<string, AttributeValue>}
	 */
	build(){
		return this.object;
	}

}