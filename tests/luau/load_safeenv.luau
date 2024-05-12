local luau = require("@lune/luau")

local testBytecode = luau.compile(
	[[
local start = os.clock()
local x
for i = 1, 1e6 do
	x = math.sqrt(i)
end
local finish = os.clock()

return finish - start]],
	{
		optimizationLevel = 2,
		coverageLevel = 0,
		debugLevel = 0,
	}
)

-- load the bytecode with different configurations
local safeCodegenFunction = luau.load(testBytecode, {
	debugName = "safeCodegenFunction",
	codegenEnabled = true,
})
local unsafeCodegenFunction = luau.load(testBytecode, {
	debugName = "unsafeCodegenFunction",
	codegenEnabled = true,
	environment = {},
	injectGlobals = true,
})
local safeFunction = luau.load(testBytecode, {
	debugName = "safeFunction",
	codegenEnabled = false,
})
local unsafeFunction = luau.load(testBytecode, {
	debugName = "unsafeFunction",
	codegenEnabled = false,
	environment = {},
	injectGlobals = true,
})

-- run the functions to get the timings
local safeCodegenTime = safeCodegenFunction()
local unsafeCodegenTime = unsafeCodegenFunction()
local safeTime = safeFunction()
local unsafeTime = unsafeFunction()

-- assert that safeCodegenTime is always twice as fast as both unsafe functions
local safeCodegenUpperBound = safeCodegenTime * 2
assert(
	unsafeCodegenTime > safeCodegenUpperBound and unsafeTime > safeCodegenUpperBound,
	"expected luau.load with codegenEnabled = true and no custom environment to use codegen"
)

-- assert that safeTime is always atleast twice as fast as both unsafe functions
local safeUpperBound = safeTime * 2
assert(
	unsafeCodegenTime > safeUpperBound and unsafeTime > safeUpperBound,
	"expected luau.load with codegenEnabled = false and no custom environment to have safeenv enabled"
)

-- normally we'd also want to check whether codegen is actually being enabled by comparing timings of safe_codegen_fn and safe_fn
-- but since we don't hae a way of checking whether the current device even supports codegen, we can't safely test this.
