import _m0 from "protobufjs/minimal";
import {
    FeedEntity,
    FeedHeader,
    FeedMessage,
    Alert,
} from "../routes/api/service/gtfs-realtime.ts";
import type {StopAlerts} from "./feeder.ts";

export const FeedMessageWithAlerts = {
    encode(message: FeedMessage & StopAlerts, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
        if (message.header !== undefined) {
            FeedHeader.encode(message.header, writer.uint32(10).fork()).ldelim();
        }
        for (const v of message.entity) {
            FeedEntity.encode(v!, writer.uint32(18).fork()).ldelim();
        }
        for (const v of message.alerts) {
            Alert.encode(v!, writer.uint32(26).fork()).ldelim();
        }
        return writer;
    },

    decode(input: _m0.Reader | Uint8Array, length?: number): FeedMessage & StopAlerts {
        const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
        let end = length === undefined ? reader.len : reader.pos + length;
        const message = this.createBaseFeedMessage();
        while (reader.pos < end) {
            const tag = reader.uint32();
            switch (tag >>> 3) {
                case 1:
                    message.header = FeedHeader.decode(reader, reader.uint32());
                    break;
                case 2:
                    message.entity.push(FeedEntity.decode(reader, reader.uint32()));
                    break;
                case 3:
                    message.alerts.push(Alert.decode(reader, reader.uint32()));
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
            }
        }
        return message;
    },

    createBaseFeedMessage(): FeedMessage & StopAlerts {
        return { header: undefined, entity: [], alerts: [] };
    }
}