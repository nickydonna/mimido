/** @typedef {import('@aws-sdk/client-dynamodb').AttributeValue} AttributeValue */

/**
 * @template {BaseEntity} T
 * @typedef EntityStaticMethods 
 * @prop {() => string} sGetTableName
 * @prop {() => import('@aws-sdk/client-dynamodb').CreateTableCommandInput}  getTableCreateInput
 * @prop {(query: Record<string, AttributeValue>) => T} fromDBItem
 */

/** @abstract */
export default class BaseEntity {
	/** @return {string} */
	static sGetTableName(){ throw new Error("Not Implemented, please override") }

	/** @return {import('@aws-sdk/client-dynamodb').CreateTableCommandInput} */
	static getTableCreateInput() { throw new Error("Not Implemented, please override") }

	/**
	 * @param {Record<string, AttributeValue>} item
	 * @return {BaseEntity}
	 */
	// eslint-disable-next-line no-unused-vars
	static fromDBItem(item) {
		throw new Error("Not Implemented, please override");
	}

	/**
	 *
	 * @param {FormData} data
	 * @return {Promise<unknown>}
	 */
	// eslint-disable-next-line no-unused-vars
	static async newFromForm(data){
		throw new Error("Not Implemented, please override");
	}

	/**
	 * @abstractRecord<string, any>
	 * @return {string}
	 */
	getTableName() { throw new Error('Not Implemented')}

	/**
	 * @abstract
	 * @return {Record<string, AttributeValue>}
	 */
	toDbItem() { throw new Error('Not Implemented')}

	/**
	 * @abstract
	 * @return { { [x:string]: any } }
	 */
	toPOJO() { throw new Error('Not Implemented')}
}