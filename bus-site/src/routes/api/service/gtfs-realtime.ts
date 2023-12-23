/* eslint-disable */
import Long from "long";
import _m0 from "protobufjs/minimal.js";

export const protobufPackage = "transit_realtime";

/**
 * The contents of a feed message.
 * A feed is a continuous stream of feed messages. Each message in the stream is
 * obtained as a response to an appropriate HTTP GET request.
 * A realtime feed is always defined with relation to an existing GTFS feed.
 * All the entity ids are resolved with respect to the GTFS feed.
 * Note that "required" and "optional" as stated in this file refer to Protocol
 * Buffer cardinality, not semantic cardinality.  See reference.md at
 * https://github.com/google/transit/tree/master/gtfs-realtime for field
 * semantic cardinality.
 */
export interface FeedMessage {
  /** Metadata about this feed and feed message. */
  header:
    | FeedHeader
    | undefined;
  /** Contents of the feed. */
  entity: FeedEntity[];
}

/** Metadata about a feed, included in feed messages. */
export interface FeedHeader {
  /**
   * Version of the feed specification.
   * The current version is 2.0.
   */
  gtfsRealtimeVersion: string;
  incrementality: FeedHeader_Incrementality;
  /**
   * This timestamp identifies the moment when the content of this feed has been
   * created (in server time). In POSIX time (i.e., number of seconds since
   * January 1st 1970 00:00:00 UTC).
   */
  timestamp: number;
}

/**
 * Determines whether the current fetch is incremental.  Currently,
 * DIFFERENTIAL mode is unsupported and behavior is unspecified for feeds
 * that use this mode.  There are discussions on the GTFS Realtime mailing
 * list around fully specifying the behavior of DIFFERENTIAL mode and the
 * documentation will be updated when those discussions are finalized.
 */
export enum FeedHeader_Incrementality {
  FULL_DATASET = 0,
  DIFFERENTIAL = 1,
  UNRECOGNIZED = -1,
}

export function feedHeader_IncrementalityFromJSON(object: any): FeedHeader_Incrementality {
  switch (object) {
    case 0:
    case "FULL_DATASET":
      return FeedHeader_Incrementality.FULL_DATASET;
    case 1:
    case "DIFFERENTIAL":
      return FeedHeader_Incrementality.DIFFERENTIAL;
    case -1:
    case "UNRECOGNIZED":
    default:
      return FeedHeader_Incrementality.UNRECOGNIZED;
  }
}

export function feedHeader_IncrementalityToJSON(object: FeedHeader_Incrementality): string {
  switch (object) {
    case FeedHeader_Incrementality.FULL_DATASET:
      return "FULL_DATASET";
    case FeedHeader_Incrementality.DIFFERENTIAL:
      return "DIFFERENTIAL";
    case FeedHeader_Incrementality.UNRECOGNIZED:
    default:
      return "UNRECOGNIZED";
  }
}

/** A definition (or update) of an entity in the transit feed. */
export interface FeedEntity {
  /**
   * The ids are used only to provide incrementality support. The id should be
   * unique within a FeedMessage. Consequent FeedMessages may contain
   * FeedEntities with the same id. In case of a DIFFERENTIAL update the new
   * FeedEntity with some id will replace the old FeedEntity with the same id
   * (or delete it - see is_deleted below).
   * The actual GTFS entities (e.g. stations, routes, trips) referenced by the
   * feed must be specified by explicit selectors (see EntitySelector below for
   * more info).
   */
  id: string;
  /**
   * Whether this entity is to be deleted. Relevant only for incremental
   * fetches.
   */
  isDeleted: boolean;
  /**
   * Data about the entity itself. Exactly one of the following fields must be
   * present (unless the entity is being deleted).
   */
  tripUpdate: TripUpdate | undefined;
  vehicle: VehiclePosition | undefined;
  alert: Alert | undefined;
}

/**
 * Realtime update of the progress of a vehicle along a trip.
 * Depending on the value of ScheduleRelationship, a TripUpdate can specify:
 * - A trip that proceeds along the schedule.
 * - A trip that proceeds along a route but has no fixed schedule.
 * - A trip that have been added or removed with regard to schedule.
 *
 * The updates can be for future, predicted arrival/departure events, or for
 * past events that already occurred.
 * Normally, updates should get more precise and more certain (see
 * uncertainty below) as the events gets closer to current time.
 * Even if that is not possible, the information for past events should be
 * precise and certain. In particular, if an update points to time in the past
 * but its update's uncertainty is not 0, the client should conclude that the
 * update is a (wrong) prediction and that the trip has not completed yet.
 *
 * Note that the update can describe a trip that is already completed.
 * To this end, it is enough to provide an update for the last stop of the trip.
 * If the time of that is in the past, the client will conclude from that that
 * the whole trip is in the past (it is possible, although inconsequential, to
 * also provide updates for preceding stops).
 * This option is most relevant for a trip that has completed ahead of schedule,
 * but according to the schedule, the trip is still proceeding at the current
 * time. Removing the updates for this trip could make the client assume
 * that the trip is still proceeding.
 * Note that the feed provider is allowed, but not required, to purge past
 * updates - this is one case where this would be practically useful.
 */
export interface TripUpdate {
  /**
   * The Trip that this message applies to. There can be at most one
   * TripUpdate entity for each actual trip instance.
   * If there is none, that means there is no prediction information available.
   * It does *not* mean that the trip is progressing according to schedule.
   */
  trip:
    | TripDescriptor
    | undefined;
  /** Additional information on the vehicle that is serving this trip. */
  vehicle:
    | VehicleDescriptor
    | undefined;
  /**
   * Updates to StopTimes for the trip (both future, i.e., predictions, and in
   * some cases, past ones, i.e., those that already happened).
   * The updates must be sorted by stop_sequence, and apply for all the
   * following stops of the trip up to the next specified one.
   *
   * Example 1:
   * For a trip with 20 stops, a StopTimeUpdate with arrival delay and departure
   * delay of 0 for stop_sequence of the current stop means that the trip is
   * exactly on time.
   *
   * Example 2:
   * For the same trip instance, 3 StopTimeUpdates are provided:
   * - delay of 5 min for stop_sequence 3
   * - delay of 1 min for stop_sequence 8
   * - delay of unspecified duration for stop_sequence 10
   * This will be interpreted as:
   * - stop_sequences 3,4,5,6,7 have delay of 5 min.
   * - stop_sequences 8,9 have delay of 1 min.
   * - stop_sequences 10,... have unknown delay.
   */
  stopTimeUpdate: TripUpdate_StopTimeUpdate[];
  /**
   * Moment at which the vehicle's real-time progress was measured. In POSIX
   * time (i.e., the number of seconds since January 1st 1970 00:00:00 UTC).
   */
  timestamp: number;
  /**
   * The current schedule deviation for the trip.  Delay should only be
   * specified when the prediction is given relative to some existing schedule
   * in GTFS.
   *
   * Delay (in seconds) can be positive (meaning that the vehicle is late) or
   * negative (meaning that the vehicle is ahead of schedule). Delay of 0
   * means that the vehicle is exactly on time.
   *
   * Delay information in StopTimeUpdates take precedent of trip-level delay
   * information, such that trip-level delay is only propagated until the next
   * stop along the trip with a StopTimeUpdate delay value specified.
   *
   * Feed providers are strongly encouraged to provide a TripUpdate.timestamp
   * value indicating when the delay value was last updated, in order to
   * evaluate the freshness of the data.
   *
   * NOTE: This field is still experimental, and subject to change. It may be
   * formally adopted in the future.
   */
  delay: number;
}

/**
 * Timing information for a single predicted event (either arrival or
 * departure).
 * Timing consists of delay and/or estimated time, and uncertainty.
 * - delay should be used when the prediction is given relative to some
 *   existing schedule in GTFS.
 * - time should be given whether there is a predicted schedule or not. If
 *   both time and delay are specified, time will take precedence
 *   (although normally, time, if given for a scheduled trip, should be
 *   equal to scheduled time in GTFS + delay).
 *
 * Uncertainty applies equally to both time and delay.
 * The uncertainty roughly specifies the expected error in true delay (but
 * note, we don't yet define its precise statistical meaning). It's possible
 * for the uncertainty to be 0, for example for trains that are driven under
 * computer timing control.
 */
export interface TripUpdate_StopTimeEvent {
  /**
   * Delay (in seconds) can be positive (meaning that the vehicle is late) or
   * negative (meaning that the vehicle is ahead of schedule). Delay of 0
   * means that the vehicle is exactly on time.
   */
  delay: number;
  /**
   * Event as absolute time.
   * In Unix time (i.e., number of seconds since January 1st 1970 00:00:00
   * UTC).
   */
  time: number;
  /**
   * If uncertainty is omitted, it is interpreted as unknown.
   * If the prediction is unknown or too uncertain, the delay (or time) field
   * should be empty. In such case, the uncertainty field is ignored.
   * To specify a completely certain prediction, set its uncertainty to 0.
   */
  uncertainty: number;
}

/**
 * Realtime update for arrival and/or departure events for a given stop on a
 * trip. Updates can be supplied for both past and future events.
 * The producer is allowed, although not required, to drop past events.
 */
export interface TripUpdate_StopTimeUpdate {
  /** Must be the same as in stop_times.txt in the corresponding GTFS feed. */
  stopSequence: number;
  /** Must be the same as in stops.txt in the corresponding GTFS feed. */
  stopId: string;
  arrival: TripUpdate_StopTimeEvent | undefined;
  departure: TripUpdate_StopTimeEvent | undefined;
  scheduleRelationship: TripUpdate_StopTimeUpdate_ScheduleRelationship;
}

/** The relation between this StopTime and the static schedule. */
export enum TripUpdate_StopTimeUpdate_ScheduleRelationship {
  /**
   * SCHEDULED - The vehicle is proceeding in accordance with its static schedule of
   * stops, although not necessarily according to the times of the schedule.
   * At least one of arrival and departure must be provided. If the schedule
   * for this stop contains both arrival and departure times then so must
   * this update.
   */
  SCHEDULED = 0,
  /**
   * SKIPPED - The stop is skipped, i.e., the vehicle will not stop at this stop.
   * Arrival and departure are optional.
   */
  SKIPPED = 1,
  /**
   * NO_DATA - No data is given for this stop. The main intention for this value is to
   * give the predictions only for part of a trip, i.e., if the last update
   * for a trip has a NO_DATA specifier, then StopTimes for the rest of the
   * stops in the trip are considered to be unspecified as well.
   * Neither arrival nor departure should be supplied.
   */
  NO_DATA = 2,
  UNRECOGNIZED = -1,
}

export function tripUpdate_StopTimeUpdate_ScheduleRelationshipFromJSON(
  object: any,
): TripUpdate_StopTimeUpdate_ScheduleRelationship {
  switch (object) {
    case 0:
    case "SCHEDULED":
      return TripUpdate_StopTimeUpdate_ScheduleRelationship.SCHEDULED;
    case 1:
    case "SKIPPED":
      return TripUpdate_StopTimeUpdate_ScheduleRelationship.SKIPPED;
    case 2:
    case "NO_DATA":
      return TripUpdate_StopTimeUpdate_ScheduleRelationship.NO_DATA;
    case -1:
    case "UNRECOGNIZED":
    default:
      return TripUpdate_StopTimeUpdate_ScheduleRelationship.UNRECOGNIZED;
  }
}

export function tripUpdate_StopTimeUpdate_ScheduleRelationshipToJSON(
  object: TripUpdate_StopTimeUpdate_ScheduleRelationship,
): string {
  switch (object) {
    case TripUpdate_StopTimeUpdate_ScheduleRelationship.SCHEDULED:
      return "SCHEDULED";
    case TripUpdate_StopTimeUpdate_ScheduleRelationship.SKIPPED:
      return "SKIPPED";
    case TripUpdate_StopTimeUpdate_ScheduleRelationship.NO_DATA:
      return "NO_DATA";
    case TripUpdate_StopTimeUpdate_ScheduleRelationship.UNRECOGNIZED:
    default:
      return "UNRECOGNIZED";
  }
}

/** Realtime positioning information for a given vehicle. */
export interface VehiclePosition {
  /**
   * The Trip that this vehicle is serving.
   * Can be empty or partial if the vehicle can not be identified with a given
   * trip instance.
   */
  trip:
    | TripDescriptor
    | undefined;
  /** Additional information on the vehicle that is serving this trip. */
  vehicle:
    | VehicleDescriptor
    | undefined;
  /** Current position of this vehicle. */
  position:
    | Position
    | undefined;
  /**
   * The stop sequence index of the current stop. The meaning of
   * current_stop_sequence (i.e., the stop that it refers to) is determined by
   * current_status.
   * If current_status is missing IN_TRANSIT_TO is assumed.
   */
  currentStopSequence: number;
  /**
   * Identifies the current stop. The value must be the same as in stops.txt in
   * the corresponding GTFS feed.
   */
  stopId: string;
  /**
   * The exact status of the vehicle with respect to the current stop.
   * Ignored if current_stop_sequence is missing.
   */
  currentStatus: VehiclePosition_VehicleStopStatus;
  /**
   * Moment at which the vehicle's position was measured. In POSIX time
   * (i.e., number of seconds since January 1st 1970 00:00:00 UTC).
   */
  timestamp: number;
  congestionLevel: VehiclePosition_CongestionLevel;
  occupancyStatus: VehiclePosition_OccupancyStatus;
}

export enum VehiclePosition_VehicleStopStatus {
  /**
   * INCOMING_AT - The vehicle is just about to arrive at the stop (on a stop
   * display, the vehicle symbol typically flashes).
   */
  INCOMING_AT = 0,
  /** STOPPED_AT - The vehicle is standing at the stop. */
  STOPPED_AT = 1,
  /** IN_TRANSIT_TO - The vehicle has departed and is in transit to the next stop. */
  IN_TRANSIT_TO = 2,
  UNRECOGNIZED = -1,
}

export function vehiclePosition_VehicleStopStatusFromJSON(object: any): VehiclePosition_VehicleStopStatus {
  switch (object) {
    case 0:
    case "INCOMING_AT":
      return VehiclePosition_VehicleStopStatus.INCOMING_AT;
    case 1:
    case "STOPPED_AT":
      return VehiclePosition_VehicleStopStatus.STOPPED_AT;
    case 2:
    case "IN_TRANSIT_TO":
      return VehiclePosition_VehicleStopStatus.IN_TRANSIT_TO;
    case -1:
    case "UNRECOGNIZED":
    default:
      return VehiclePosition_VehicleStopStatus.UNRECOGNIZED;
  }
}

export function vehiclePosition_VehicleStopStatusToJSON(object: VehiclePosition_VehicleStopStatus): string {
  switch (object) {
    case VehiclePosition_VehicleStopStatus.INCOMING_AT:
      return "INCOMING_AT";
    case VehiclePosition_VehicleStopStatus.STOPPED_AT:
      return "STOPPED_AT";
    case VehiclePosition_VehicleStopStatus.IN_TRANSIT_TO:
      return "IN_TRANSIT_TO";
    case VehiclePosition_VehicleStopStatus.UNRECOGNIZED:
    default:
      return "UNRECOGNIZED";
  }
}

/** Congestion level that is affecting this vehicle. */
export enum VehiclePosition_CongestionLevel {
  UNKNOWN_CONGESTION_LEVEL = 0,
  RUNNING_SMOOTHLY = 1,
  STOP_AND_GO = 2,
  CONGESTION = 3,
  /** SEVERE_CONGESTION - People leaving their cars. */
  SEVERE_CONGESTION = 4,
  UNRECOGNIZED = -1,
}

export function vehiclePosition_CongestionLevelFromJSON(object: any): VehiclePosition_CongestionLevel {
  switch (object) {
    case 0:
    case "UNKNOWN_CONGESTION_LEVEL":
      return VehiclePosition_CongestionLevel.UNKNOWN_CONGESTION_LEVEL;
    case 1:
    case "RUNNING_SMOOTHLY":
      return VehiclePosition_CongestionLevel.RUNNING_SMOOTHLY;
    case 2:
    case "STOP_AND_GO":
      return VehiclePosition_CongestionLevel.STOP_AND_GO;
    case 3:
    case "CONGESTION":
      return VehiclePosition_CongestionLevel.CONGESTION;
    case 4:
    case "SEVERE_CONGESTION":
      return VehiclePosition_CongestionLevel.SEVERE_CONGESTION;
    case -1:
    case "UNRECOGNIZED":
    default:
      return VehiclePosition_CongestionLevel.UNRECOGNIZED;
  }
}

export function vehiclePosition_CongestionLevelToJSON(object: VehiclePosition_CongestionLevel): string {
  switch (object) {
    case VehiclePosition_CongestionLevel.UNKNOWN_CONGESTION_LEVEL:
      return "UNKNOWN_CONGESTION_LEVEL";
    case VehiclePosition_CongestionLevel.RUNNING_SMOOTHLY:
      return "RUNNING_SMOOTHLY";
    case VehiclePosition_CongestionLevel.STOP_AND_GO:
      return "STOP_AND_GO";
    case VehiclePosition_CongestionLevel.CONGESTION:
      return "CONGESTION";
    case VehiclePosition_CongestionLevel.SEVERE_CONGESTION:
      return "SEVERE_CONGESTION";
    case VehiclePosition_CongestionLevel.UNRECOGNIZED:
    default:
      return "UNRECOGNIZED";
  }
}

/**
 * The degree of passenger occupancy of the vehicle. This field is still
 * experimental, and subject to change. It may be formally adopted in the
 * future.
 */
export enum VehiclePosition_OccupancyStatus {
  /**
   * EMPTY - The vehicle is considered empty by most measures, and has few or no
   * passengers onboard, but is still accepting passengers.
   */
  EMPTY = 0,
  /**
   * MANY_SEATS_AVAILABLE - The vehicle has a relatively large percentage of seats available.
   * What percentage of free seats out of the total seats available is to be
   * considered large enough to fall into this category is determined at the
   * discretion of the producer.
   */
  MANY_SEATS_AVAILABLE = 1,
  /**
   * FEW_SEATS_AVAILABLE - The vehicle has a relatively small percentage of seats available.
   * What percentage of free seats out of the total seats available is to be
   * considered small enough to fall into this category is determined at the
   * discretion of the feed producer.
   */
  FEW_SEATS_AVAILABLE = 2,
  /** STANDING_ROOM_ONLY - The vehicle can currently accommodate only standing passengers. */
  STANDING_ROOM_ONLY = 3,
  /**
   * CRUSHED_STANDING_ROOM_ONLY - The vehicle can currently accommodate only standing passengers
   * and has limited space for them.
   */
  CRUSHED_STANDING_ROOM_ONLY = 4,
  /**
   * FULL - The vehicle is considered full by most measures, but may still be
   * allowing passengers to board.
   */
  FULL = 5,
  /** NOT_ACCEPTING_PASSENGERS - The vehicle is not accepting additional passengers. */
  NOT_ACCEPTING_PASSENGERS = 6,
  UNRECOGNIZED = -1,
}

export function vehiclePosition_OccupancyStatusFromJSON(object: any): VehiclePosition_OccupancyStatus {
  switch (object) {
    case 0:
    case "EMPTY":
      return VehiclePosition_OccupancyStatus.EMPTY;
    case 1:
    case "MANY_SEATS_AVAILABLE":
      return VehiclePosition_OccupancyStatus.MANY_SEATS_AVAILABLE;
    case 2:
    case "FEW_SEATS_AVAILABLE":
      return VehiclePosition_OccupancyStatus.FEW_SEATS_AVAILABLE;
    case 3:
    case "STANDING_ROOM_ONLY":
      return VehiclePosition_OccupancyStatus.STANDING_ROOM_ONLY;
    case 4:
    case "CRUSHED_STANDING_ROOM_ONLY":
      return VehiclePosition_OccupancyStatus.CRUSHED_STANDING_ROOM_ONLY;
    case 5:
    case "FULL":
      return VehiclePosition_OccupancyStatus.FULL;
    case 6:
    case "NOT_ACCEPTING_PASSENGERS":
      return VehiclePosition_OccupancyStatus.NOT_ACCEPTING_PASSENGERS;
    case -1:
    case "UNRECOGNIZED":
    default:
      return VehiclePosition_OccupancyStatus.UNRECOGNIZED;
  }
}

export function vehiclePosition_OccupancyStatusToJSON(object: VehiclePosition_OccupancyStatus): string {
  switch (object) {
    case VehiclePosition_OccupancyStatus.EMPTY:
      return "EMPTY";
    case VehiclePosition_OccupancyStatus.MANY_SEATS_AVAILABLE:
      return "MANY_SEATS_AVAILABLE";
    case VehiclePosition_OccupancyStatus.FEW_SEATS_AVAILABLE:
      return "FEW_SEATS_AVAILABLE";
    case VehiclePosition_OccupancyStatus.STANDING_ROOM_ONLY:
      return "STANDING_ROOM_ONLY";
    case VehiclePosition_OccupancyStatus.CRUSHED_STANDING_ROOM_ONLY:
      return "CRUSHED_STANDING_ROOM_ONLY";
    case VehiclePosition_OccupancyStatus.FULL:
      return "FULL";
    case VehiclePosition_OccupancyStatus.NOT_ACCEPTING_PASSENGERS:
      return "NOT_ACCEPTING_PASSENGERS";
    case VehiclePosition_OccupancyStatus.UNRECOGNIZED:
    default:
      return "UNRECOGNIZED";
  }
}

/** An alert, indicating some sort of incident in the public transit network. */
export interface Alert {
  /**
   * Time when the alert should be shown to the user. If missing, the
   * alert will be shown as long as it appears in the feed.
   * If multiple ranges are given, the alert will be shown during all of them.
   */
  activePeriod: TimeRange[];
  /** Entities whose users we should notify of this alert. */
  informedEntity: EntitySelector[];
  cause: Alert_Cause;
  effect: Alert_Effect;
  /** The URL which provides additional information about the alert. */
  url:
    | TranslatedString
    | undefined;
  /** Alert header. Contains a short summary of the alert text as plain-text. */
  headerText:
    | TranslatedString
    | undefined;
  /**
   * Full description for the alert as plain-text. The information in the
   * description should add to the information of the header.
   */
  descriptionText: TranslatedString | undefined;
}

/** Cause of this alert. */
export enum Alert_Cause {
  UNKNOWN_CAUSE = 1,
  /** OTHER_CAUSE - Not machine-representable. */
  OTHER_CAUSE = 2,
  TECHNICAL_PROBLEM = 3,
  /** STRIKE - Public transit agency employees stopped working. */
  STRIKE = 4,
  /** DEMONSTRATION - People are blocking the streets. */
  DEMONSTRATION = 5,
  ACCIDENT = 6,
  HOLIDAY = 7,
  WEATHER = 8,
  MAINTENANCE = 9,
  CONSTRUCTION = 10,
  POLICE_ACTIVITY = 11,
  MEDICAL_EMERGENCY = 12,
  UNRECOGNIZED = -1,
}

export function alert_CauseFromJSON(object: any): Alert_Cause {
  switch (object) {
    case 1:
    case "UNKNOWN_CAUSE":
      return Alert_Cause.UNKNOWN_CAUSE;
    case 2:
    case "OTHER_CAUSE":
      return Alert_Cause.OTHER_CAUSE;
    case 3:
    case "TECHNICAL_PROBLEM":
      return Alert_Cause.TECHNICAL_PROBLEM;
    case 4:
    case "STRIKE":
      return Alert_Cause.STRIKE;
    case 5:
    case "DEMONSTRATION":
      return Alert_Cause.DEMONSTRATION;
    case 6:
    case "ACCIDENT":
      return Alert_Cause.ACCIDENT;
    case 7:
    case "HOLIDAY":
      return Alert_Cause.HOLIDAY;
    case 8:
    case "WEATHER":
      return Alert_Cause.WEATHER;
    case 9:
    case "MAINTENANCE":
      return Alert_Cause.MAINTENANCE;
    case 10:
    case "CONSTRUCTION":
      return Alert_Cause.CONSTRUCTION;
    case 11:
    case "POLICE_ACTIVITY":
      return Alert_Cause.POLICE_ACTIVITY;
    case 12:
    case "MEDICAL_EMERGENCY":
      return Alert_Cause.MEDICAL_EMERGENCY;
    case -1:
    case "UNRECOGNIZED":
    default:
      return Alert_Cause.UNRECOGNIZED;
  }
}

export function alert_CauseToJSON(object: Alert_Cause): string {
  switch (object) {
    case Alert_Cause.UNKNOWN_CAUSE:
      return "UNKNOWN_CAUSE";
    case Alert_Cause.OTHER_CAUSE:
      return "OTHER_CAUSE";
    case Alert_Cause.TECHNICAL_PROBLEM:
      return "TECHNICAL_PROBLEM";
    case Alert_Cause.STRIKE:
      return "STRIKE";
    case Alert_Cause.DEMONSTRATION:
      return "DEMONSTRATION";
    case Alert_Cause.ACCIDENT:
      return "ACCIDENT";
    case Alert_Cause.HOLIDAY:
      return "HOLIDAY";
    case Alert_Cause.WEATHER:
      return "WEATHER";
    case Alert_Cause.MAINTENANCE:
      return "MAINTENANCE";
    case Alert_Cause.CONSTRUCTION:
      return "CONSTRUCTION";
    case Alert_Cause.POLICE_ACTIVITY:
      return "POLICE_ACTIVITY";
    case Alert_Cause.MEDICAL_EMERGENCY:
      return "MEDICAL_EMERGENCY";
    case Alert_Cause.UNRECOGNIZED:
    default:
      return "UNRECOGNIZED";
  }
}

/** What is the effect of this problem on the affected entity. */
export enum Alert_Effect {
  NO_SERVICE = 1,
  REDUCED_SERVICE = 2,
  /**
   * SIGNIFICANT_DELAYS - We don't care about INsignificant delays: they are hard to detect, have
   * little impact on the user, and would clutter the results as they are too
   * frequent.
   */
  SIGNIFICANT_DELAYS = 3,
  DETOUR = 4,
  ADDITIONAL_SERVICE = 5,
  MODIFIED_SERVICE = 6,
  OTHER_EFFECT = 7,
  UNKNOWN_EFFECT = 8,
  STOP_MOVED = 9,
  UNRECOGNIZED = -1,
}

export function alert_EffectFromJSON(object: any): Alert_Effect {
  switch (object) {
    case 1:
    case "NO_SERVICE":
      return Alert_Effect.NO_SERVICE;
    case 2:
    case "REDUCED_SERVICE":
      return Alert_Effect.REDUCED_SERVICE;
    case 3:
    case "SIGNIFICANT_DELAYS":
      return Alert_Effect.SIGNIFICANT_DELAYS;
    case 4:
    case "DETOUR":
      return Alert_Effect.DETOUR;
    case 5:
    case "ADDITIONAL_SERVICE":
      return Alert_Effect.ADDITIONAL_SERVICE;
    case 6:
    case "MODIFIED_SERVICE":
      return Alert_Effect.MODIFIED_SERVICE;
    case 7:
    case "OTHER_EFFECT":
      return Alert_Effect.OTHER_EFFECT;
    case 8:
    case "UNKNOWN_EFFECT":
      return Alert_Effect.UNKNOWN_EFFECT;
    case 9:
    case "STOP_MOVED":
      return Alert_Effect.STOP_MOVED;
    case -1:
    case "UNRECOGNIZED":
    default:
      return Alert_Effect.UNRECOGNIZED;
  }
}

export function alert_EffectToJSON(object: Alert_Effect): string {
  switch (object) {
    case Alert_Effect.NO_SERVICE:
      return "NO_SERVICE";
    case Alert_Effect.REDUCED_SERVICE:
      return "REDUCED_SERVICE";
    case Alert_Effect.SIGNIFICANT_DELAYS:
      return "SIGNIFICANT_DELAYS";
    case Alert_Effect.DETOUR:
      return "DETOUR";
    case Alert_Effect.ADDITIONAL_SERVICE:
      return "ADDITIONAL_SERVICE";
    case Alert_Effect.MODIFIED_SERVICE:
      return "MODIFIED_SERVICE";
    case Alert_Effect.OTHER_EFFECT:
      return "OTHER_EFFECT";
    case Alert_Effect.UNKNOWN_EFFECT:
      return "UNKNOWN_EFFECT";
    case Alert_Effect.STOP_MOVED:
      return "STOP_MOVED";
    case Alert_Effect.UNRECOGNIZED:
    default:
      return "UNRECOGNIZED";
  }
}

/**
 * A time interval. The interval is considered active at time 't' if 't' is
 * greater than or equal to the start time and less than the end time.
 */
export interface TimeRange {
  /**
   * Start time, in POSIX time (i.e., number of seconds since January 1st 1970
   * 00:00:00 UTC).
   * If missing, the interval starts at minus infinity.
   */
  start: number;
  /**
   * End time, in POSIX time (i.e., number of seconds since January 1st 1970
   * 00:00:00 UTC).
   * If missing, the interval ends at plus infinity.
   */
  end: number;
}

/** A position. */
export interface Position {
  /** Degrees North, in the WGS-84 coordinate system. */
  latitude: number;
  /** Degrees East, in the WGS-84 coordinate system. */
  longitude: number;
  /**
   * Bearing, in degrees, clockwise from North, i.e., 0 is North and 90 is East.
   * This can be the compass bearing, or the direction towards the next stop
   * or intermediate location.
   * This should not be direction deduced from the sequence of previous
   * positions, which can be computed from previous data.
   */
  bearing: number;
  /** Odometer value, in meters. */
  odometer: number;
  /** Momentary speed measured by the vehicle, in meters per second. */
  speed: number;
}

/**
 * A descriptor that identifies an instance of a GTFS trip, or all instances of
 * a trip along a route.
 * - To specify a single trip instance, the trip_id (and if necessary,
 *   start_time) is set. If route_id is also set, then it should be same as one
 *   that the given trip corresponds to.
 * - To specify all the trips along a given route, only the route_id should be
 *   set. Note that if the trip_id is not known, then stop sequence ids in
 *   TripUpdate are not sufficient, and stop_ids must be provided as well. In
 *   addition, absolute arrival/departure times must be provided.
 */
export interface TripDescriptor {
  /**
   * The trip_id from the GTFS feed that this selector refers to.
   * For non frequency-based trips, this field is enough to uniquely identify
   * the trip. For frequency-based trip, start_time and start_date might also be
   * necessary.
   */
  tripId: string;
  /** The route_id from the GTFS that this selector refers to. */
  routeId: string;
  /**
   * The direction_id from the GTFS feed trips.txt file, indicating the
   * direction of travel for trips this selector refers to. This field is
   * still experimental, and subject to change. It may be formally adopted in
   * the future.
   */
  directionId: number;
  /**
   * The initially scheduled start time of this trip instance.
   * When the trip_id corresponds to a non-frequency-based trip, this field
   * should either be omitted or be equal to the value in the GTFS feed. When
   * the trip_id corresponds to a frequency-based trip, the start_time must be
   * specified for trip updates and vehicle positions. If the trip corresponds
   * to exact_times=1 GTFS record, then start_time must be some multiple
   * (including zero) of headway_secs later than frequencies.txt start_time for
   * the corresponding time period. If the trip corresponds to exact_times=0,
   * then its start_time may be arbitrary, and is initially expected to be the
   * first departure of the trip. Once established, the start_time of this
   * frequency-based trip should be considered immutable, even if the first
   * departure time changes -- that time change may instead be reflected in a
   * StopTimeUpdate.
   * Format and semantics of the field is same as that of
   * GTFS/frequencies.txt/start_time, e.g., 11:15:35 or 25:15:35.
   */
  startTime: string;
  /**
   * The scheduled start date of this trip instance.
   * Must be provided to disambiguate trips that are so late as to collide with
   * a scheduled trip on a next day. For example, for a train that departs 8:00
   * and 20:00 every day, and is 12 hours late, there would be two distinct
   * trips on the same time.
   * This field can be provided but is not mandatory for schedules in which such
   * collisions are impossible - for example, a service running on hourly
   * schedule where a vehicle that is one hour late is not considered to be
   * related to schedule anymore.
   * In YYYYMMDD format.
   */
  startDate: string;
  scheduleRelationship: TripDescriptor_ScheduleRelationship;
}

/**
 * The relation between this trip and the static schedule. If a trip is done
 * in accordance with temporary schedule, not reflected in GTFS, then it
 * shouldn't be marked as SCHEDULED, but likely as ADDED.
 */
export enum TripDescriptor_ScheduleRelationship {
  /**
   * SCHEDULED - Trip that is running in accordance with its GTFS schedule, or is close
   * enough to the scheduled trip to be associated with it.
   */
  SCHEDULED = 0,
  /**
   * ADDED - An extra trip that was added in addition to a running schedule, for
   * example, to replace a broken vehicle or to respond to sudden passenger
   * load.
   */
  ADDED = 1,
  /**
   * UNSCHEDULED - A trip that is running with no schedule associated to it, for example, if
   * there is no schedule at all.
   */
  UNSCHEDULED = 2,
  /** CANCELED - A trip that existed in the schedule but was removed. */
  CANCELED = 3,
  UNRECOGNIZED = -1,
}

export function tripDescriptor_ScheduleRelationshipFromJSON(object: any): TripDescriptor_ScheduleRelationship {
  switch (object) {
    case 0:
    case "SCHEDULED":
      return TripDescriptor_ScheduleRelationship.SCHEDULED;
    case 1:
    case "ADDED":
      return TripDescriptor_ScheduleRelationship.ADDED;
    case 2:
    case "UNSCHEDULED":
      return TripDescriptor_ScheduleRelationship.UNSCHEDULED;
    case 3:
    case "CANCELED":
      return TripDescriptor_ScheduleRelationship.CANCELED;
    case -1:
    case "UNRECOGNIZED":
    default:
      return TripDescriptor_ScheduleRelationship.UNRECOGNIZED;
  }
}

export function tripDescriptor_ScheduleRelationshipToJSON(object: TripDescriptor_ScheduleRelationship): string {
  switch (object) {
    case TripDescriptor_ScheduleRelationship.SCHEDULED:
      return "SCHEDULED";
    case TripDescriptor_ScheduleRelationship.ADDED:
      return "ADDED";
    case TripDescriptor_ScheduleRelationship.UNSCHEDULED:
      return "UNSCHEDULED";
    case TripDescriptor_ScheduleRelationship.CANCELED:
      return "CANCELED";
    case TripDescriptor_ScheduleRelationship.UNRECOGNIZED:
    default:
      return "UNRECOGNIZED";
  }
}

/** Identification information for the vehicle performing the trip. */
export interface VehicleDescriptor {
  /**
   * Internal system identification of the vehicle. Should be unique per
   * vehicle, and can be used for tracking the vehicle as it proceeds through
   * the system.
   */
  id: string;
  /**
   * User visible label, i.e., something that must be shown to the passenger to
   * help identify the correct vehicle.
   */
  label: string;
  /** The license plate of the vehicle. */
  licensePlate: string;
}

/** A selector for an entity in a GTFS feed. */
export interface EntitySelector {
  /**
   * The values of the fields should correspond to the appropriate fields in the
   * GTFS feed.
   * At least one specifier must be given. If several are given, then the
   * matching has to apply to all the given specifiers.
   */
  agencyId: string;
  routeId: string;
  /** corresponds to route_type in GTFS. */
  routeType: number;
  trip: TripDescriptor | undefined;
  stopId: string;
}

/**
 * An internationalized message containing per-language versions of a snippet of
 * text or a URL.
 * One of the strings from a message will be picked up. The resolution proceeds
 * as follows:
 * 1. If the UI language matches the language code of a translation,
 *    the first matching translation is picked.
 * 2. If a default UI language (e.g., English) matches the language code of a
 *    translation, the first matching translation is picked.
 * 3. If some translation has an unspecified language code, that translation is
 *    picked.
 */
export interface TranslatedString {
  /** At least one translation must be provided. */
  translation: TranslatedString_Translation[];
}

export interface TranslatedString_Translation {
  /** A UTF-8 string containing the message. */
  text: string;
  /**
   * BCP-47 language code. Can be omitted if the language is unknown or if
   * no i18n is done at all for the feed. At most one translation is
   * allowed to have an unspecified language tag.
   */
  language: string;
}

function createBaseFeedMessage(): FeedMessage {
  return { header: undefined, entity: [] };
}

export const FeedMessage = {
  encode(message: FeedMessage, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.header !== undefined) {
      FeedHeader.encode(message.header, writer.uint32(10).fork()).ldelim();
    }
    for (const v of message.entity) {
      FeedEntity.encode(v!, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): FeedMessage {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseFeedMessage();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.header = FeedHeader.decode(reader, reader.uint32());
          break;
        case 2:
          message.entity.push(FeedEntity.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): FeedMessage {
    return {
      header: isSet(object.header) ? FeedHeader.fromJSON(object.header) : undefined,
      entity: Array.isArray(object?.entity) ? object.entity.map((e: any) => FeedEntity.fromJSON(e)) : [],
    };
  },

  toJSON(message: FeedMessage): unknown {
    const obj: any = {};
    message.header !== undefined && (obj.header = message.header ? FeedHeader.toJSON(message.header) : undefined);
    if (message.entity) {
      obj.entity = message.entity.map((e) => e ? FeedEntity.toJSON(e) : undefined);
    } else {
      obj.entity = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FeedMessage>, I>>(object: I): FeedMessage {
    const message = createBaseFeedMessage();
    message.header = (object.header !== undefined && object.header !== null)
      ? FeedHeader.fromPartial(object.header)
      : undefined;
    message.entity = object.entity?.map((e) => FeedEntity.fromPartial(e)) || [];
    return message;
  },
};

function createBaseFeedHeader(): FeedHeader {
  return { gtfsRealtimeVersion: "", incrementality: 0, timestamp: 0 };
}

export const FeedHeader = {
  encode(message: FeedHeader, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.gtfsRealtimeVersion !== "") {
      writer.uint32(10).string(message.gtfsRealtimeVersion);
    }
    if (message.incrementality !== 0) {
      writer.uint32(16).int32(message.incrementality);
    }
    if (message.timestamp !== 0) {
      writer.uint32(24).uint64(message.timestamp);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): FeedHeader {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseFeedHeader();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.gtfsRealtimeVersion = reader.string();
          break;
        case 2:
          message.incrementality = reader.int32() as any;
          break;
        case 3:
          message.timestamp = longToNumber(reader.uint64() as Long);
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): FeedHeader {
    return {
      gtfsRealtimeVersion: isSet(object.gtfsRealtimeVersion) ? String(object.gtfsRealtimeVersion) : "",
      incrementality: isSet(object.incrementality) ? feedHeader_IncrementalityFromJSON(object.incrementality) : 0,
      timestamp: isSet(object.timestamp) ? Number(object.timestamp) : 0,
    };
  },

  toJSON(message: FeedHeader): unknown {
    const obj: any = {};
    message.gtfsRealtimeVersion !== undefined && (obj.gtfsRealtimeVersion = message.gtfsRealtimeVersion);
    message.incrementality !== undefined &&
      (obj.incrementality = feedHeader_IncrementalityToJSON(message.incrementality));
    message.timestamp !== undefined && (obj.timestamp = Math.round(message.timestamp));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FeedHeader>, I>>(object: I): FeedHeader {
    const message = createBaseFeedHeader();
    message.gtfsRealtimeVersion = object.gtfsRealtimeVersion ?? "";
    message.incrementality = object.incrementality ?? 0;
    message.timestamp = object.timestamp ?? 0;
    return message;
  },
};

function createBaseFeedEntity(): FeedEntity {
  return { id: "", isDeleted: false, tripUpdate: undefined, vehicle: undefined, alert: undefined };
}

export const FeedEntity = {
  encode(message: FeedEntity, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.id !== "") {
      writer.uint32(10).string(message.id);
    }
    if (message.isDeleted === true) {
      writer.uint32(16).bool(message.isDeleted);
    }
    if (message.tripUpdate !== undefined) {
      TripUpdate.encode(message.tripUpdate, writer.uint32(26).fork()).ldelim();
    }
    if (message.vehicle !== undefined) {
      VehiclePosition.encode(message.vehicle, writer.uint32(34).fork()).ldelim();
    }
    if (message.alert !== undefined) {
      Alert.encode(message.alert, writer.uint32(42).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): FeedEntity {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseFeedEntity();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = reader.string();
          break;
        case 2:
          message.isDeleted = reader.bool();
          break;
        case 3:
          message.tripUpdate = TripUpdate.decode(reader, reader.uint32());
          break;
        case 4:
          message.vehicle = VehiclePosition.decode(reader, reader.uint32());
          break;
        case 5:
          message.alert = Alert.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): FeedEntity {
    return {
      id: isSet(object.id) ? String(object.id) : "",
      isDeleted: isSet(object.isDeleted) ? Boolean(object.isDeleted) : false,
      tripUpdate: isSet(object.tripUpdate) ? TripUpdate.fromJSON(object.tripUpdate) : undefined,
      vehicle: isSet(object.vehicle) ? VehiclePosition.fromJSON(object.vehicle) : undefined,
      alert: isSet(object.alert) ? Alert.fromJSON(object.alert) : undefined,
    };
  },

  toJSON(message: FeedEntity): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    message.isDeleted !== undefined && (obj.isDeleted = message.isDeleted);
    message.tripUpdate !== undefined &&
      (obj.tripUpdate = message.tripUpdate ? TripUpdate.toJSON(message.tripUpdate) : undefined);
    message.vehicle !== undefined &&
      (obj.vehicle = message.vehicle ? VehiclePosition.toJSON(message.vehicle) : undefined);
    message.alert !== undefined && (obj.alert = message.alert ? Alert.toJSON(message.alert) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FeedEntity>, I>>(object: I): FeedEntity {
    const message = createBaseFeedEntity();
    message.id = object.id ?? "";
    message.isDeleted = object.isDeleted ?? false;
    message.tripUpdate = (object.tripUpdate !== undefined && object.tripUpdate !== null)
      ? TripUpdate.fromPartial(object.tripUpdate)
      : undefined;
    message.vehicle = (object.vehicle !== undefined && object.vehicle !== null)
      ? VehiclePosition.fromPartial(object.vehicle)
      : undefined;
    message.alert = (object.alert !== undefined && object.alert !== null) ? Alert.fromPartial(object.alert) : undefined;
    return message;
  },
};

function createBaseTripUpdate(): TripUpdate {
  return { trip: undefined, vehicle: undefined, stopTimeUpdate: [], timestamp: 0, delay: 0 };
}

export const TripUpdate = {
  encode(message: TripUpdate, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.trip !== undefined) {
      TripDescriptor.encode(message.trip, writer.uint32(10).fork()).ldelim();
    }
    if (message.vehicle !== undefined) {
      VehicleDescriptor.encode(message.vehicle, writer.uint32(26).fork()).ldelim();
    }
    for (const v of message.stopTimeUpdate) {
      TripUpdate_StopTimeUpdate.encode(v!, writer.uint32(18).fork()).ldelim();
    }
    if (message.timestamp !== 0) {
      writer.uint32(32).uint64(message.timestamp);
    }
    if (message.delay !== 0) {
      writer.uint32(40).int32(message.delay);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TripUpdate {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTripUpdate();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.trip = TripDescriptor.decode(reader, reader.uint32());
          break;
        case 3:
          message.vehicle = VehicleDescriptor.decode(reader, reader.uint32());
          break;
        case 2:
          message.stopTimeUpdate.push(TripUpdate_StopTimeUpdate.decode(reader, reader.uint32()));
          break;
        case 4:
          message.timestamp = longToNumber(reader.uint64() as Long);
          break;
        case 5:
          message.delay = reader.int32();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): TripUpdate {
    return {
      trip: isSet(object.trip) ? TripDescriptor.fromJSON(object.trip) : undefined,
      vehicle: isSet(object.vehicle) ? VehicleDescriptor.fromJSON(object.vehicle) : undefined,
      stopTimeUpdate: Array.isArray(object?.stopTimeUpdate)
        ? object.stopTimeUpdate.map((e: any) => TripUpdate_StopTimeUpdate.fromJSON(e))
        : [],
      timestamp: isSet(object.timestamp) ? Number(object.timestamp) : 0,
      delay: isSet(object.delay) ? Number(object.delay) : 0,
    };
  },

  toJSON(message: TripUpdate): unknown {
    const obj: any = {};
    message.trip !== undefined && (obj.trip = message.trip ? TripDescriptor.toJSON(message.trip) : undefined);
    message.vehicle !== undefined &&
      (obj.vehicle = message.vehicle ? VehicleDescriptor.toJSON(message.vehicle) : undefined);
    if (message.stopTimeUpdate) {
      obj.stopTimeUpdate = message.stopTimeUpdate.map((e) => e ? TripUpdate_StopTimeUpdate.toJSON(e) : undefined);
    } else {
      obj.stopTimeUpdate = [];
    }
    message.timestamp !== undefined && (obj.timestamp = Math.round(message.timestamp));
    message.delay !== undefined && (obj.delay = Math.round(message.delay));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<TripUpdate>, I>>(object: I): TripUpdate {
    const message = createBaseTripUpdate();
    message.trip = (object.trip !== undefined && object.trip !== null)
      ? TripDescriptor.fromPartial(object.trip)
      : undefined;
    message.vehicle = (object.vehicle !== undefined && object.vehicle !== null)
      ? VehicleDescriptor.fromPartial(object.vehicle)
      : undefined;
    message.stopTimeUpdate = object.stopTimeUpdate?.map((e) => TripUpdate_StopTimeUpdate.fromPartial(e)) || [];
    message.timestamp = object.timestamp ?? 0;
    message.delay = object.delay ?? 0;
    return message;
  },
};

function createBaseTripUpdate_StopTimeEvent(): TripUpdate_StopTimeEvent {
  return { delay: 0, time: 0, uncertainty: 0 };
}

export const TripUpdate_StopTimeEvent = {
  encode(message: TripUpdate_StopTimeEvent, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.delay !== 0) {
      writer.uint32(8).int32(message.delay);
    }
    if (message.time !== 0) {
      writer.uint32(16).int64(message.time);
    }
    if (message.uncertainty !== 0) {
      writer.uint32(24).int32(message.uncertainty);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TripUpdate_StopTimeEvent {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTripUpdate_StopTimeEvent();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.delay = reader.int32();
          break;
        case 2:
          message.time = longToNumber(reader.int64() as Long);
          break;
        case 3:
          message.uncertainty = reader.int32();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): TripUpdate_StopTimeEvent {
    return {
      delay: isSet(object.delay) ? Number(object.delay) : 0,
      time: isSet(object.time) ? Number(object.time) : 0,
      uncertainty: isSet(object.uncertainty) ? Number(object.uncertainty) : 0,
    };
  },

  toJSON(message: TripUpdate_StopTimeEvent): unknown {
    const obj: any = {};
    message.delay !== undefined && (obj.delay = Math.round(message.delay));
    message.time !== undefined && (obj.time = Math.round(message.time));
    message.uncertainty !== undefined && (obj.uncertainty = Math.round(message.uncertainty));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<TripUpdate_StopTimeEvent>, I>>(object: I): TripUpdate_StopTimeEvent {
    const message = createBaseTripUpdate_StopTimeEvent();
    message.delay = object.delay ?? 0;
    message.time = object.time ?? 0;
    message.uncertainty = object.uncertainty ?? 0;
    return message;
  },
};

function createBaseTripUpdate_StopTimeUpdate(): TripUpdate_StopTimeUpdate {
  return { stopSequence: 0, stopId: "", arrival: undefined, departure: undefined, scheduleRelationship: 0 };
}

export const TripUpdate_StopTimeUpdate = {
  encode(message: TripUpdate_StopTimeUpdate, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.stopSequence !== 0) {
      writer.uint32(8).uint32(message.stopSequence);
    }
    if (message.stopId !== "") {
      writer.uint32(34).string(message.stopId);
    }
    if (message.arrival !== undefined) {
      TripUpdate_StopTimeEvent.encode(message.arrival, writer.uint32(18).fork()).ldelim();
    }
    if (message.departure !== undefined) {
      TripUpdate_StopTimeEvent.encode(message.departure, writer.uint32(26).fork()).ldelim();
    }
    if (message.scheduleRelationship !== 0) {
      writer.uint32(40).int32(message.scheduleRelationship);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TripUpdate_StopTimeUpdate {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTripUpdate_StopTimeUpdate();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.stopSequence = reader.uint32();
          break;
        case 4:
          message.stopId = reader.string();
          break;
        case 2:
          message.arrival = TripUpdate_StopTimeEvent.decode(reader, reader.uint32());
          break;
        case 3:
          message.departure = TripUpdate_StopTimeEvent.decode(reader, reader.uint32());
          break;
        case 5:
          message.scheduleRelationship = reader.int32() as any;
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): TripUpdate_StopTimeUpdate {
    return {
      stopSequence: isSet(object.stopSequence) ? Number(object.stopSequence) : 0,
      stopId: isSet(object.stopId) ? String(object.stopId) : "",
      arrival: isSet(object.arrival) ? TripUpdate_StopTimeEvent.fromJSON(object.arrival) : undefined,
      departure: isSet(object.departure) ? TripUpdate_StopTimeEvent.fromJSON(object.departure) : undefined,
      scheduleRelationship: isSet(object.scheduleRelationship)
        ? tripUpdate_StopTimeUpdate_ScheduleRelationshipFromJSON(object.scheduleRelationship)
        : 0,
    };
  },

  toJSON(message: TripUpdate_StopTimeUpdate): unknown {
    const obj: any = {};
    message.stopSequence !== undefined && (obj.stopSequence = Math.round(message.stopSequence));
    message.stopId !== undefined && (obj.stopId = message.stopId);
    message.arrival !== undefined &&
      (obj.arrival = message.arrival ? TripUpdate_StopTimeEvent.toJSON(message.arrival) : undefined);
    message.departure !== undefined &&
      (obj.departure = message.departure ? TripUpdate_StopTimeEvent.toJSON(message.departure) : undefined);
    message.scheduleRelationship !== undefined &&
      (obj.scheduleRelationship = tripUpdate_StopTimeUpdate_ScheduleRelationshipToJSON(message.scheduleRelationship));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<TripUpdate_StopTimeUpdate>, I>>(object: I): TripUpdate_StopTimeUpdate {
    const message = createBaseTripUpdate_StopTimeUpdate();
    message.stopSequence = object.stopSequence ?? 0;
    message.stopId = object.stopId ?? "";
    message.arrival = (object.arrival !== undefined && object.arrival !== null)
      ? TripUpdate_StopTimeEvent.fromPartial(object.arrival)
      : undefined;
    message.departure = (object.departure !== undefined && object.departure !== null)
      ? TripUpdate_StopTimeEvent.fromPartial(object.departure)
      : undefined;
    message.scheduleRelationship = object.scheduleRelationship ?? 0;
    return message;
  },
};

function createBaseVehiclePosition(): VehiclePosition {
  return {
    trip: undefined,
    vehicle: undefined,
    position: undefined,
    currentStopSequence: 0,
    stopId: "",
    currentStatus: 0,
    timestamp: 0,
    congestionLevel: 0,
    occupancyStatus: 0,
  };
}

export const VehiclePosition = {
  encode(message: VehiclePosition, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.trip !== undefined) {
      TripDescriptor.encode(message.trip, writer.uint32(10).fork()).ldelim();
    }
    if (message.vehicle !== undefined) {
      VehicleDescriptor.encode(message.vehicle, writer.uint32(66).fork()).ldelim();
    }
    if (message.position !== undefined) {
      Position.encode(message.position, writer.uint32(18).fork()).ldelim();
    }
    if (message.currentStopSequence !== 0) {
      writer.uint32(24).uint32(message.currentStopSequence);
    }
    if (message.stopId !== "") {
      writer.uint32(58).string(message.stopId);
    }
    if (message.currentStatus !== 0) {
      writer.uint32(32).int32(message.currentStatus);
    }
    if (message.timestamp !== 0) {
      writer.uint32(40).uint64(message.timestamp);
    }
    if (message.congestionLevel !== 0) {
      writer.uint32(48).int32(message.congestionLevel);
    }
    if (message.occupancyStatus !== 0) {
      writer.uint32(72).int32(message.occupancyStatus);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): VehiclePosition {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseVehiclePosition();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.trip = TripDescriptor.decode(reader, reader.uint32());
          break;
        case 8:
          message.vehicle = VehicleDescriptor.decode(reader, reader.uint32());
          break;
        case 2:
          message.position = Position.decode(reader, reader.uint32());
          break;
        case 3:
          message.currentStopSequence = reader.uint32();
          break;
        case 7:
          message.stopId = reader.string();
          break;
        case 4:
          message.currentStatus = reader.int32() as any;
          break;
        case 5:
          message.timestamp = longToNumber(reader.uint64() as Long);
          break;
        case 6:
          message.congestionLevel = reader.int32() as any;
          break;
        case 9:
          message.occupancyStatus = reader.int32() as any;
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): VehiclePosition {
    return {
      trip: isSet(object.trip) ? TripDescriptor.fromJSON(object.trip) : undefined,
      vehicle: isSet(object.vehicle) ? VehicleDescriptor.fromJSON(object.vehicle) : undefined,
      position: isSet(object.position) ? Position.fromJSON(object.position) : undefined,
      currentStopSequence: isSet(object.currentStopSequence) ? Number(object.currentStopSequence) : 0,
      stopId: isSet(object.stopId) ? String(object.stopId) : "",
      currentStatus: isSet(object.currentStatus) ? vehiclePosition_VehicleStopStatusFromJSON(object.currentStatus) : 0,
      timestamp: isSet(object.timestamp) ? Number(object.timestamp) : 0,
      congestionLevel: isSet(object.congestionLevel)
        ? vehiclePosition_CongestionLevelFromJSON(object.congestionLevel)
        : 0,
      occupancyStatus: isSet(object.occupancyStatus)
        ? vehiclePosition_OccupancyStatusFromJSON(object.occupancyStatus)
        : 0,
    };
  },

  toJSON(message: VehiclePosition): unknown {
    const obj: any = {};
    message.trip !== undefined && (obj.trip = message.trip ? TripDescriptor.toJSON(message.trip) : undefined);
    message.vehicle !== undefined &&
      (obj.vehicle = message.vehicle ? VehicleDescriptor.toJSON(message.vehicle) : undefined);
    message.position !== undefined && (obj.position = message.position ? Position.toJSON(message.position) : undefined);
    message.currentStopSequence !== undefined && (obj.currentStopSequence = Math.round(message.currentStopSequence));
    message.stopId !== undefined && (obj.stopId = message.stopId);
    message.currentStatus !== undefined &&
      (obj.currentStatus = vehiclePosition_VehicleStopStatusToJSON(message.currentStatus));
    message.timestamp !== undefined && (obj.timestamp = Math.round(message.timestamp));
    message.congestionLevel !== undefined &&
      (obj.congestionLevel = vehiclePosition_CongestionLevelToJSON(message.congestionLevel));
    message.occupancyStatus !== undefined &&
      (obj.occupancyStatus = vehiclePosition_OccupancyStatusToJSON(message.occupancyStatus));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<VehiclePosition>, I>>(object: I): VehiclePosition {
    const message = createBaseVehiclePosition();
    message.trip = (object.trip !== undefined && object.trip !== null)
      ? TripDescriptor.fromPartial(object.trip)
      : undefined;
    message.vehicle = (object.vehicle !== undefined && object.vehicle !== null)
      ? VehicleDescriptor.fromPartial(object.vehicle)
      : undefined;
    message.position = (object.position !== undefined && object.position !== null)
      ? Position.fromPartial(object.position)
      : undefined;
    message.currentStopSequence = object.currentStopSequence ?? 0;
    message.stopId = object.stopId ?? "";
    message.currentStatus = object.currentStatus ?? 0;
    message.timestamp = object.timestamp ?? 0;
    message.congestionLevel = object.congestionLevel ?? 0;
    message.occupancyStatus = object.occupancyStatus ?? 0;
    return message;
  },
};

function createBaseAlert(): Alert {
  return {
    activePeriod: [],
    informedEntity: [],
    cause: 1,
    effect: 1,
    url: undefined,
    headerText: undefined,
    descriptionText: undefined,
  };
}

export const Alert = {
  encode(message: Alert, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    for (const v of message.activePeriod) {
      TimeRange.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    for (const v of message.informedEntity) {
      EntitySelector.encode(v!, writer.uint32(42).fork()).ldelim();
    }
    if (message.cause !== 1) {
      writer.uint32(48).int32(message.cause);
    }
    if (message.effect !== 1) {
      writer.uint32(56).int32(message.effect);
    }
    if (message.url !== undefined) {
      TranslatedString.encode(message.url, writer.uint32(66).fork()).ldelim();
    }
    if (message.headerText !== undefined) {
      TranslatedString.encode(message.headerText, writer.uint32(82).fork()).ldelim();
    }
    if (message.descriptionText !== undefined) {
      TranslatedString.encode(message.descriptionText, writer.uint32(90).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Alert {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseAlert();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.activePeriod.push(TimeRange.decode(reader, reader.uint32()));
          break;
        case 5:
          message.informedEntity.push(EntitySelector.decode(reader, reader.uint32()));
          break;
        case 6:
          message.cause = reader.int32() as any;
          break;
        case 7:
          message.effect = reader.int32() as any;
          break;
        case 8:
          message.url = TranslatedString.decode(reader, reader.uint32());
          break;
        case 10:
          message.headerText = TranslatedString.decode(reader, reader.uint32());
          break;
        case 11:
          message.descriptionText = TranslatedString.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Alert {
    return {
      activePeriod: Array.isArray(object?.activePeriod)
        ? object.activePeriod.map((e: any) => TimeRange.fromJSON(e))
        : [],
      informedEntity: Array.isArray(object?.informedEntity)
        ? object.informedEntity.map((e: any) => EntitySelector.fromJSON(e))
        : [],
      cause: isSet(object.cause) ? alert_CauseFromJSON(object.cause) : 1,
      effect: isSet(object.effect) ? alert_EffectFromJSON(object.effect) : 1,
      url: isSet(object.url) ? TranslatedString.fromJSON(object.url) : undefined,
      headerText: isSet(object.headerText) ? TranslatedString.fromJSON(object.headerText) : undefined,
      descriptionText: isSet(object.descriptionText) ? TranslatedString.fromJSON(object.descriptionText) : undefined,
    };
  },

  toJSON(message: Alert): unknown {
    const obj: any = {};
    if (message.activePeriod) {
      obj.activePeriod = message.activePeriod.map((e) => e ? TimeRange.toJSON(e) : undefined);
    } else {
      obj.activePeriod = [];
    }
    if (message.informedEntity) {
      obj.informedEntity = message.informedEntity.map((e) => e ? EntitySelector.toJSON(e) : undefined);
    } else {
      obj.informedEntity = [];
    }
    message.cause !== undefined && (obj.cause = alert_CauseToJSON(message.cause));
    message.effect !== undefined && (obj.effect = alert_EffectToJSON(message.effect));
    message.url !== undefined && (obj.url = message.url ? TranslatedString.toJSON(message.url) : undefined);
    message.headerText !== undefined &&
      (obj.headerText = message.headerText ? TranslatedString.toJSON(message.headerText) : undefined);
    message.descriptionText !== undefined &&
      (obj.descriptionText = message.descriptionText ? TranslatedString.toJSON(message.descriptionText) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Alert>, I>>(object: I): Alert {
    const message = createBaseAlert();
    message.activePeriod = object.activePeriod?.map((e) => TimeRange.fromPartial(e)) || [];
    message.informedEntity = object.informedEntity?.map((e) => EntitySelector.fromPartial(e)) || [];
    message.cause = object.cause ?? 1;
    message.effect = object.effect ?? 1;
    message.url = (object.url !== undefined && object.url !== null)
      ? TranslatedString.fromPartial(object.url)
      : undefined;
    message.headerText = (object.headerText !== undefined && object.headerText !== null)
      ? TranslatedString.fromPartial(object.headerText)
      : undefined;
    message.descriptionText = (object.descriptionText !== undefined && object.descriptionText !== null)
      ? TranslatedString.fromPartial(object.descriptionText)
      : undefined;
    return message;
  },
};

function createBaseTimeRange(): TimeRange {
  return { start: 0, end: 0 };
}

export const TimeRange = {
  encode(message: TimeRange, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.start !== 0) {
      writer.uint32(8).uint64(message.start);
    }
    if (message.end !== 0) {
      writer.uint32(16).uint64(message.end);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TimeRange {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTimeRange();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.start = longToNumber(reader.uint64() as Long);
          break;
        case 2:
          message.end = longToNumber(reader.uint64() as Long);
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): TimeRange {
    return { start: isSet(object.start) ? Number(object.start) : 0, end: isSet(object.end) ? Number(object.end) : 0 };
  },

  toJSON(message: TimeRange): unknown {
    const obj: any = {};
    message.start !== undefined && (obj.start = Math.round(message.start));
    message.end !== undefined && (obj.end = Math.round(message.end));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<TimeRange>, I>>(object: I): TimeRange {
    const message = createBaseTimeRange();
    message.start = object.start ?? 0;
    message.end = object.end ?? 0;
    return message;
  },
};

function createBasePosition(): Position {
  return { latitude: 0, longitude: 0, bearing: 0, odometer: 0, speed: 0 };
}

export const Position = {
  encode(message: Position, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.latitude !== 0) {
      writer.uint32(13).float(message.latitude);
    }
    if (message.longitude !== 0) {
      writer.uint32(21).float(message.longitude);
    }
    if (message.bearing !== 0) {
      writer.uint32(29).float(message.bearing);
    }
    if (message.odometer !== 0) {
      writer.uint32(33).double(message.odometer);
    }
    if (message.speed !== 0) {
      writer.uint32(45).float(message.speed);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Position {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePosition();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.latitude = reader.float();
          break;
        case 2:
          message.longitude = reader.float();
          break;
        case 3:
          message.bearing = reader.float();
          break;
        case 4:
          message.odometer = reader.double();
          break;
        case 5:
          message.speed = reader.float();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Position {
    return {
      latitude: isSet(object.latitude) ? Number(object.latitude) : 0,
      longitude: isSet(object.longitude) ? Number(object.longitude) : 0,
      bearing: isSet(object.bearing) ? Number(object.bearing) : 0,
      odometer: isSet(object.odometer) ? Number(object.odometer) : 0,
      speed: isSet(object.speed) ? Number(object.speed) : 0,
    };
  },

  toJSON(message: Position): unknown {
    const obj: any = {};
    message.latitude !== undefined && (obj.latitude = message.latitude);
    message.longitude !== undefined && (obj.longitude = message.longitude);
    message.bearing !== undefined && (obj.bearing = message.bearing);
    message.odometer !== undefined && (obj.odometer = message.odometer);
    message.speed !== undefined && (obj.speed = message.speed);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Position>, I>>(object: I): Position {
    const message = createBasePosition();
    message.latitude = object.latitude ?? 0;
    message.longitude = object.longitude ?? 0;
    message.bearing = object.bearing ?? 0;
    message.odometer = object.odometer ?? 0;
    message.speed = object.speed ?? 0;
    return message;
  },
};

function createBaseTripDescriptor(): TripDescriptor {
  return { tripId: "", routeId: "", directionId: 0, startTime: "", startDate: "", scheduleRelationship: 0 };
}

export const TripDescriptor = {
  encode(message: TripDescriptor, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.tripId !== "") {
      writer.uint32(10).string(message.tripId);
    }
    if (message.routeId !== "") {
      writer.uint32(42).string(message.routeId);
    }
    if (message.directionId !== 0) {
      writer.uint32(48).uint32(message.directionId);
    }
    if (message.startTime !== "") {
      writer.uint32(18).string(message.startTime);
    }
    if (message.startDate !== "") {
      writer.uint32(26).string(message.startDate);
    }
    if (message.scheduleRelationship !== 0) {
      writer.uint32(32).int32(message.scheduleRelationship);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TripDescriptor {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTripDescriptor();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.tripId = reader.string();
          break;
        case 5:
          message.routeId = reader.string();
          break;
        case 6:
          message.directionId = reader.uint32();
          break;
        case 2:
          message.startTime = reader.string();
          break;
        case 3:
          message.startDate = reader.string();
          break;
        case 4:
          message.scheduleRelationship = reader.int32() as any;
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): TripDescriptor {
    return {
      tripId: isSet(object.tripId) ? String(object.tripId) : "",
      routeId: isSet(object.routeId) ? String(object.routeId) : "",
      directionId: isSet(object.directionId) ? Number(object.directionId) : 0,
      startTime: isSet(object.startTime) ? String(object.startTime) : "",
      startDate: isSet(object.startDate) ? String(object.startDate) : "",
      scheduleRelationship: isSet(object.scheduleRelationship)
        ? tripDescriptor_ScheduleRelationshipFromJSON(object.scheduleRelationship)
        : 0,
    };
  },

  toJSON(message: TripDescriptor): unknown {
    const obj: any = {};
    message.tripId !== undefined && (obj.tripId = message.tripId);
    message.routeId !== undefined && (obj.routeId = message.routeId);
    message.directionId !== undefined && (obj.directionId = Math.round(message.directionId));
    message.startTime !== undefined && (obj.startTime = message.startTime);
    message.startDate !== undefined && (obj.startDate = message.startDate);
    message.scheduleRelationship !== undefined &&
      (obj.scheduleRelationship = tripDescriptor_ScheduleRelationshipToJSON(message.scheduleRelationship));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<TripDescriptor>, I>>(object: I): TripDescriptor {
    const message = createBaseTripDescriptor();
    message.tripId = object.tripId ?? "";
    message.routeId = object.routeId ?? "";
    message.directionId = object.directionId ?? 0;
    message.startTime = object.startTime ?? "";
    message.startDate = object.startDate ?? "";
    message.scheduleRelationship = object.scheduleRelationship ?? 0;
    return message;
  },
};

function createBaseVehicleDescriptor(): VehicleDescriptor {
  return { id: "", label: "", licensePlate: "" };
}

export const VehicleDescriptor = {
  encode(message: VehicleDescriptor, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.id !== "") {
      writer.uint32(10).string(message.id);
    }
    if (message.label !== "") {
      writer.uint32(18).string(message.label);
    }
    if (message.licensePlate !== "") {
      writer.uint32(26).string(message.licensePlate);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): VehicleDescriptor {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseVehicleDescriptor();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = reader.string();
          break;
        case 2:
          message.label = reader.string();
          break;
        case 3:
          message.licensePlate = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): VehicleDescriptor {
    return {
      id: isSet(object.id) ? String(object.id) : "",
      label: isSet(object.label) ? String(object.label) : "",
      licensePlate: isSet(object.licensePlate) ? String(object.licensePlate) : "",
    };
  },

  toJSON(message: VehicleDescriptor): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    message.label !== undefined && (obj.label = message.label);
    message.licensePlate !== undefined && (obj.licensePlate = message.licensePlate);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<VehicleDescriptor>, I>>(object: I): VehicleDescriptor {
    const message = createBaseVehicleDescriptor();
    message.id = object.id ?? "";
    message.label = object.label ?? "";
    message.licensePlate = object.licensePlate ?? "";
    return message;
  },
};

function createBaseEntitySelector(): EntitySelector {
  return { agencyId: "", routeId: "", routeType: 0, trip: undefined, stopId: "" };
}

export const EntitySelector = {
  encode(message: EntitySelector, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.agencyId !== "") {
      writer.uint32(10).string(message.agencyId);
    }
    if (message.routeId !== "") {
      writer.uint32(18).string(message.routeId);
    }
    if (message.routeType !== 0) {
      writer.uint32(24).int32(message.routeType);
    }
    if (message.trip !== undefined) {
      TripDescriptor.encode(message.trip, writer.uint32(34).fork()).ldelim();
    }
    if (message.stopId !== "") {
      writer.uint32(42).string(message.stopId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): EntitySelector {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseEntitySelector();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.agencyId = reader.string();
          break;
        case 2:
          message.routeId = reader.string();
          break;
        case 3:
          message.routeType = reader.int32();
          break;
        case 4:
          message.trip = TripDescriptor.decode(reader, reader.uint32());
          break;
        case 5:
          message.stopId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): EntitySelector {
    return {
      agencyId: isSet(object.agencyId) ? String(object.agencyId) : "",
      routeId: isSet(object.routeId) ? String(object.routeId) : "",
      routeType: isSet(object.routeType) ? Number(object.routeType) : 0,
      trip: isSet(object.trip) ? TripDescriptor.fromJSON(object.trip) : undefined,
      stopId: isSet(object.stopId) ? String(object.stopId) : "",
    };
  },

  toJSON(message: EntitySelector): unknown {
    const obj: any = {};
    message.agencyId !== undefined && (obj.agencyId = message.agencyId);
    message.routeId !== undefined && (obj.routeId = message.routeId);
    message.routeType !== undefined && (obj.routeType = Math.round(message.routeType));
    message.trip !== undefined && (obj.trip = message.trip ? TripDescriptor.toJSON(message.trip) : undefined);
    message.stopId !== undefined && (obj.stopId = message.stopId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<EntitySelector>, I>>(object: I): EntitySelector {
    const message = createBaseEntitySelector();
    message.agencyId = object.agencyId ?? "";
    message.routeId = object.routeId ?? "";
    message.routeType = object.routeType ?? 0;
    message.trip = (object.trip !== undefined && object.trip !== null)
      ? TripDescriptor.fromPartial(object.trip)
      : undefined;
    message.stopId = object.stopId ?? "";
    return message;
  },
};

function createBaseTranslatedString(): TranslatedString {
  return { translation: [] };
}

export const TranslatedString = {
  encode(message: TranslatedString, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    for (const v of message.translation) {
      TranslatedString_Translation.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TranslatedString {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTranslatedString();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.translation.push(TranslatedString_Translation.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): TranslatedString {
    return {
      translation: Array.isArray(object?.translation)
        ? object.translation.map((e: any) => TranslatedString_Translation.fromJSON(e))
        : [],
    };
  },

  toJSON(message: TranslatedString): unknown {
    const obj: any = {};
    if (message.translation) {
      obj.translation = message.translation.map((e) => e ? TranslatedString_Translation.toJSON(e) : undefined);
    } else {
      obj.translation = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<TranslatedString>, I>>(object: I): TranslatedString {
    const message = createBaseTranslatedString();
    message.translation = object.translation?.map((e) => TranslatedString_Translation.fromPartial(e)) || [];
    return message;
  },
};

function createBaseTranslatedString_Translation(): TranslatedString_Translation {
  return { text: "", language: "" };
}

export const TranslatedString_Translation = {
  encode(message: TranslatedString_Translation, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.text !== "") {
      writer.uint32(10).string(message.text);
    }
    if (message.language !== "") {
      writer.uint32(18).string(message.language);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TranslatedString_Translation {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTranslatedString_Translation();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.text = reader.string();
          break;
        case 2:
          message.language = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): TranslatedString_Translation {
    return {
      text: isSet(object.text) ? String(object.text) : "",
      language: isSet(object.language) ? String(object.language) : "",
    };
  },

  toJSON(message: TranslatedString_Translation): unknown {
    const obj: any = {};
    message.text !== undefined && (obj.text = message.text);
    message.language !== undefined && (obj.language = message.language);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<TranslatedString_Translation>, I>>(object: I): TranslatedString_Translation {
    const message = createBaseTranslatedString_Translation();
    message.text = object.text ?? "";
    message.language = object.language ?? "";
    return message;
  },
};

declare var self: any | undefined;
declare var window: any | undefined;
declare var global: any | undefined;
var tsProtoGlobalThis: any = (() => {
  if (typeof globalThis !== "undefined") {
    return globalThis;
  }
  if (typeof self !== "undefined") {
    return self;
  }
  if (typeof window !== "undefined") {
    return window;
  }
  if (typeof global !== "undefined") {
    return global;
  }
  throw "Unable to locate global object";
})();

type Builtin = Date | Function | Uint8Array | string | number | boolean | undefined;

export type DeepPartial<T> = T extends Builtin ? T
  : T extends Array<infer U> ? Array<DeepPartial<U>> : T extends ReadonlyArray<infer U> ? ReadonlyArray<DeepPartial<U>>
  : T extends {} ? { [K in keyof T]?: DeepPartial<T[K]> }
  : Partial<T>;

type KeysOfUnion<T> = T extends T ? keyof T : never;
export type Exact<P, I extends P> = P extends Builtin ? P
  : P & { [K in keyof P]: Exact<P[K], I[K]> } & { [K in Exclude<keyof I, KeysOfUnion<P>>]: never };

function longToNumber(long: Long): number {
  if (long.gt(Number.MAX_SAFE_INTEGER)) {
    throw new tsProtoGlobalThis.Error("Value is larger than Number.MAX_SAFE_INTEGER");
  }
  return long.toNumber();
}

if (_m0.util.Long !== Long) {
  _m0.util.Long = Long as any;
  _m0.configure();
}

function isSet(value: any): boolean {
  return value !== null && value !== undefined;
}
