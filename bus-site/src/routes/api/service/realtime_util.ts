import type {FeedEntity} from "./gtfs-realtime";
import {type DateObjectUnits, DateTime} from "luxon";

export type Position = {
    x: number,
    y: number
}

export function findNearestLinePoint(s1: Position, s2: Position, point: Position): Position {
    const m = (s2.y - s1.y) / (s2.x - s1.x)
    const c = s1.y - m*s1.x
    const xp = (point.y * m + point.x - m*c) / (m**2 + 1)
    const yp = m*xp + c
    return {x: xp, y: yp}
}

export function distanceBetween(s1: Position, s2: Position) {
    return Math.sqrt((s2.x - s1.x) ** 2 + (s2.y - s1.y) ** 2)
}

export function findPctBetween(s1: Position, s2: Position, point: Position) {
    const linePoint = findNearestLinePoint(s1, s2, point)
    return Math.max(0, 1 - (distanceBetween(s1, linePoint) / distanceBetween(s1, s2)))
}

export function notNull(value: FeedEntity | null): value is FeedEntity {
    return value !== null
}

export function format_gtfs_time(time: DateTime): string {
    return time.toFormat("HH:mm:ss")
}

export function format_gtfs_date(time: DateTime): string {
    return time.toFormat("yyyyMMdd")
}

export const ZERO_DAY: DateObjectUnits = {year: 1970, ordinal: 1}
export const ZERO_TIME: DateObjectUnits = {hour: 0, minute: 0, second: 0, millisecond: 0}
export const FULL_TIME: DateObjectUnits = {hour: 23, minute: 59, second: 59, millisecond: 999}

export function sqlToLuxon(time: number) {
    return DateTime.fromSeconds(time, {zone: "GMT"})
}

export function relativeTo(date: DateTime, time: DateTime) {
    return time.toSeconds() - date.set(ZERO_TIME).toSeconds()
}

export function dayDiff(from: DateTime, to: DateTime) {
    return to.set({hour: 0, minute: 0, second: 0, millisecond: 0})
        .diff(from.set({hour: 0, minute: 0, second: 0, millisecond: 0}),['days'])
        .get("days")
}

export function minIndex(arr: any[]) {
    let lowest = 0
    for(let i = 0; i < arr.length; i++) {
        if(arr[i] < arr[lowest]) lowest = i
    }
    return lowest
}

export function merge<T>(records: Record<string, T[]>[]): Record<string, T[]> {
    let result: Record<string, T[]> = {}
    for(let record of records) {
        for(let [key, value] of Object.entries(record)) {
            if(result[key] === undefined) {
                result[key] = value.slice()
            } else {
                result[key].push(...value)
            }
        }
    }
    return result
}