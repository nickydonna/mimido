import {
	CreateTableCommand,
	DeleteTableCommand,
	GetItemCommand,
	ListTablesCommand, PutItemCommand
} from '@aws-sdk/client-dynamodb';

/** @typedef {import ('$lib/server/db/base-entity').default} BaseEntity */
/** @typedef {import('@aws-sdk/client-dynamodb').PutItemCommandOutput} PutItemCommandOutput */
/**
 * @template {BaseEntity} T
 * @typedef {import ('$lib/server/db/base-entity').EntityStaticMethods<T>} EntityStaticMethods
 * */
export default class EntityManager {
	/**
	 *
	 * @param {import('@aws-sdk/client-dynamodb').DynamoDBClient} client
	 */
	constructor(client) {
		/** @type {import('@aws-sdk/client-dynamodb').DynamoDBClient} */
		this.client = client;
	}

	/**
	 * @template {BaseEntity} T
	 * @param {EntityStaticMethods<T>} klass
	 * @param {boolean} recreate
	 */
	async createTable(klass, recreate = false) {
		const tables = await this.client.send(new ListTablesCommand({}));

		let tableExists = tables.TableNames?.some(t => t === klass.sGetTableName());

		if (tableExists && recreate) {
			await this.client.send(new DeleteTableCommand({
				TableName: klass.sGetTableName(),
			}));
			tableExists = false;
		}

		if (tableExists) {
			return;
		}

		const command = new CreateTableCommand(klass.getTableCreateInput());

		return await this.client.send(command);
	}

	/**
	 * @template {BaseEntity} T
	 * @param {EntityStaticMethods<T>} klass
	 * @param {Record<string, AttributeValue>} keyQuery
	 * @return {Promise<T | undefined>}
	 */
	async findById(klass, keyQuery) {
		const query = new GetItemCommand({
			TableName: klass.sGetTableName(),
			Key: keyQuery,
		});

		const res = await this.client.send(query);
		if (res.Item) {
			return klass.fromDBItem(res.Item);
		}
		return undefined;
	}

	/**
	 * @template {BaseEntity} T
	 * @param {T} item
	 * @return {Promise<PutItemCommandOutput>}
	 */
	async create(item) {
		const cmd = new PutItemCommand({
			TableName: item.getTableName(),
			Item: item.toDbItem()
		});

		return this.client.send(cmd);
	}
}