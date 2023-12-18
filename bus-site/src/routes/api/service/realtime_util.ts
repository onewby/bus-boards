import type {FeedEntity} from "./gtfs-realtime";
import type {DateTime} from "luxon";

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

export function findNearestSegmentPoint(s1: Position, s2: Position, point: Position): Position {
    const linePoint = findNearestLinePoint(s1, s2, point)
    const exteriorMetric = (point.x - s1.x) / (s2.x - s1.x)
    if (exteriorMetric < 0) return s1
    if (exteriorMetric > 1) return s2
    return linePoint
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