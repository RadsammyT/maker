workspace "maker"
	configurations {"Debug", "Release"}
	
    filter "configurations:Debug"
        defines { "DEBUG" }
        symbols "On"

    filter "configurations:Release"
        defines { "NDEBUG" }
        optimize "Full"

	targetdir "_bin/%{cfg.buildcfg}"

	project("maker")
		kind "ConsoleApp"
		location "_build"
		targetdir "_bin/%{cfg.buildcfg}"

		files {"src/*.c", "src/*.cpp", "src/*.h", "src/*.hpp"}
		includedirs { "./", "src", "include"}

		vpaths 
		{
		  ["Header Files/*"] = { "include/**.h",  "include/**.hpp", "src/**.h", "src/**.hpp", "**.h", "**.hpp"},
		  ["Source Files/*"] = {"src/**.c", "src/**.cpp","**.c", "**.cpp"},
		}

		cppdialect "C++20"
