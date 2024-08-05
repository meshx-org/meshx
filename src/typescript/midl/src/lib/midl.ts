import { Decoder, Encoder } from "."

export type Layout = {
    get inlineSize(): number
}

export type Encodable<T> = {
    encode(encoder: Encoder, value: T, offset: number, depth: number): void
} & Layout

export type Decodable<T> = {
    decode(decoder: Decoder, offset: number, depth: number): T
} & Layout
