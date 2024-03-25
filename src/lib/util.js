const IMPORTANCE_STRINGS = ['Sub-Zero', 'Very Low', 'Low', undefined, 'Mid', 'High', 'Very High']
const URGENCY_STRINGS = [undefined, 'Soon', 'Next Up', 'Why are you not doing it']
const LOAD_STRINGS = [undefined, 'Mid', 'Hard', 'Fat Rolling']

/**
 * @param {string | undefined} str
 * @param {string | undefined} sufix
 * @return {string}
 */
function withSufix(str, sufix) {
  if (!str) return '';
  if (!sufix) return str;
  return `${str} ${sufix}`;
}

/**
 * 
 * @param {Array<string | undefined>} list 
 * @returns {(value: number, sufix?: string) => string}
 */
function getString(list) {
  return (n, sufix) =>  withSufix(list[n], sufix)
}

/**
 * @param {number} importance
 * @param {string | undefined} [sufix]
 * @return {string}
 */
export const importanceToString = (importance, sufix) => getString(IMPORTANCE_STRINGS)(importance + 3, sufix)
export const urgencyToString = getString(URGENCY_STRINGS)
export const loadToString = getString(LOAD_STRINGS)

/**
 * TODO Ask tessy
 * @param {import("./parser").ParsedEventSchema} event 
 */
export function getRanking(event) {
  const { urgency, load, importance } = event;
  return urgency + load + importance;

}
