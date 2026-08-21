import * as assert from 'assert';
import * as index from '../../index';

async function testMulti(actual: string | undefined, expected: string | undefined) {
    assert.deepStrictEqual(actual, expected);
}

describe("applesoft,integer", async function () {
    
    it("remstk-fp1", async function () {
        const addr = 0xf8;
        let info_list = index.get_one_and_split(addr);
        if (!info_list)
            assert.fail("address not found"); 
        assert.equal(info_list.length, 2);
        await testMulti(info_list[0].ctx, "Applesoft");
        await testMulti(info_list[1].ctx, "Integer BASIC");
        await testMulti(info_list[0].type, "byte value");
        await testMulti(info_list[1].type, "float32");
        await testMulti(info_list[0].label, "REMSTK");
        await testMulti(info_list[1].label, "FP1");
    });
    it("errpos-curlin", async function () {
        const addr = 0xdc;
        let info_list = index.get_one_and_split(addr);
        if (!info_list)
            assert.fail("address not found"); 
        assert.equal(info_list.length, 2);
        await testMulti(info_list[0].ctx, "Applesoft");
        await testMulti(info_list[1].ctx, "Integer BASIC");
        await testMulti(info_list[0].type, "word");
        await testMulti(info_list[1].type, "word");
        await testMulti(info_list[0].label, "ERRPOS");
        await testMulti(info_list[1].label, "CURLIN");
    });
});

describe("no-split", async function () {
    
    it("scan-input", async function () {
        const addr = 0xff70;
        let info_list = index.get_one_and_split(addr);
        if (!info_list)
            assert.fail("address not found");
        assert.equal(info_list.length, 1);
        await testMulti(info_list[0].type, "ROM routine");
    });
    it("concat", async function () {
        const addr = 0xe597;
        let info_list = index.get_one_and_split(addr);
        if (!info_list)
            assert.fail("address not found");
        assert.equal(info_list.length, 1);
        await testMulti(info_list[0].type, "ROM routine");
        await testMulti(info_list[0].subctx, "parsing & string");
    });
});

describe("splits-but-one-ctx", async function () {
    
    it("banks-stepper", async function () {
        const addr = 0xc080;
        let info_list = index.get_one_and_split(addr);
        if (!info_list)
            assert.fail("address not found");
        assert.equal(info_list.length, 1);
        await testMulti(info_list[0].brief, "RDRAM | stepper 0");
    });
});

