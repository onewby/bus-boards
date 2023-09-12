import { writable } from 'svelte/store'
import {browser} from "$app/environment";
import type {SearchResult} from "./api.type";

const LSK_STOPS = "STOPS"
// @ts-ignore
export const starredStops: Writable<SearchResult[]> = writable(browser && localStorage.getItem(LSK_STOPS) !== null ? JSON.parse(localStorage.getItem(LSK_STOPS)) : [])
starredStops.subscribe((arr: SearchResult[]) => {
    if(browser) localStorage.setItem(LSK_STOPS, JSON.stringify(arr))
})