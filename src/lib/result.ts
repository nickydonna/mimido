import type { Result } from '../bindings';

export function isOk<T, E>(result: Result<T, E>): result is { status: 'ok', data: T } {
  return result.status === 'ok'
}

export function isErr<T, E>(result: Result<T, E>): result is { status: 'error', error: E } {
  return !isOk(result)
}

export function unwrap<T, E>(result: Result<T, E>): T {
  if (isErr(result)) {
    throw result.error;
  }
  return result.data;
}

export function ok<T, E = string>(data: T): Result<T, E> {
  return { status: 'ok', data }
}

export function err<E, T = void>(error: E): Result<T, E> {
  return { status: 'error', error }
}

export function map<A, B, E>(result: Result<A, E>, fn: (a: A) => B): Result<B, E> {
  if (isErr(result)) {
    return result;
  }
  return ok(fn(result.data))
}
