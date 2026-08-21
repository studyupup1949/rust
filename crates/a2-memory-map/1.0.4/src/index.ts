import * as memmap from "./map.json"

export interface AddressInfo {
    brief: string | undefined,
    ctx: string | undefined,
    desc: string,
    label: string | undefined,
    note: string | undefined,
    subctx: string | undefined,
    type: string
}

function get_contextual_str(str: string, count: number): string {
    const contexts = str.split('|');
    if (count < contexts.length)
        return contexts[count].trim();
    return str;
}

function get_contextual_str_maybe(str: string | undefined, count: number): string | undefined {
    if (!str)
        return str;
    const contexts = str.split('|');
    if (count < contexts.length)
        return contexts[count].trim();
    return str;
}

function get_contextual_info(info: AddressInfo, count: number): AddressInfo {
    return {
        brief: get_contextual_str_maybe(info.brief,count),
        ctx: get_contextual_str_maybe(info.ctx,count),
        desc: get_contextual_str(info.desc,count),
        subctx: get_contextual_str_maybe(info.subctx,count),
        label: get_contextual_str_maybe(info.label,count),
        note: get_contextual_str_maybe(info.note,count),
        type: get_contextual_str(info.type,count)
    }
}

/**
 * Gather the database into a map, logic strings are unprocessed.
 * @returns Map from address strings to AddressInfo objects.
 */
export function get_all(): Map<string, AddressInfo> {
    let m = new Map<string, AddressInfo>();
    for (const [key, obj] of Object.entries(memmap)) {
        m.set(key, {
            brief: 'brief' in obj ? obj.brief : undefined,
            ctx: 'ctx' in obj ? obj.ctx : undefined,
            desc: obj.desc,
            label: 'label' in obj ? obj.label : undefined,
            note: 'note' in obj ? obj.note : undefined,
            subctx: 'subctx' in obj ? obj.subctx : undefined,
            type: obj.type
        });
    }
    return m;
}

/**
 * Get data for one address
 * @param addr address in the range -32767 to 65535
 * @returns AddressInfo object or undefined. Fields may contain logic operators.
 */
export function get_one(addr: number): AddressInfo | undefined {
    let pos_addr = addr < 0 ? addr + 2 ** 16 : addr;
    const hex = '0x' + pos_addr.toString(16).padStart(4, '0');
    return Object(memmap)[hex];
}

/**
 * Get data for one address and split by context
 * @param addr Array of AddressInfo objects, one for each context, or undefined
 */
export function get_one_and_split(addr: number): Array<AddressInfo> | undefined {
    const obj = get_one(addr);
    if (obj) {
        let ans = new Array<AddressInfo>();
        const num = obj.ctx ? obj.ctx.split('|').length : 1;
        // if one context do not split anything
        if (num == 1) {
            ans.push(obj);
            return ans;
        }
        // if multi-context, split all fields on `|`
        for (let count = 0; count < num; count++) {
            ans.push(get_contextual_info(obj, count));
        }
        return ans;
    }
    return undefined
}