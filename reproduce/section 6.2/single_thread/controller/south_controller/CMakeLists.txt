cmake_minimum_required(VERSION 3.5)
project(upf_driver)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_C_COMPILER /usr/bin/gcc-11)
set(CMAKE_CXX_COMPILER /usr/bin/g++-11)

set(TESTDATADIR "")
set(SDE_INSTALL $ENV{SDE_INSTALL})
set(SDE_INSTALL_INCLUDE ${SDE_INSTALL}/include)
set(SDE_INSTALL_LIBS ${SDE_INSTALL}/lib)
include_directories(${SDE_INSTALL_INCLUDE})
link_directories(${SDE_INSTALL_LIBS})

# set(CMAKE_INSTALL_RPATH $ORIGIN/../lib)
# file( GLOB APP_SOURCES *.cpp *.c *.cc)
# foreach( sourcefile ${APP_SOURCES} )
#     get_filename_component(sourcename ${sourcefile} NAME_WE)
#     add_executable( ${sourcename} ${sourcefile} )
#     target_link_libraries( ${sourcename} PUBLIC bfutils target_sys clish driver pthread)
#     #target_include_directories( ${sourcename}_example ${SDE_INSTALL_INCLUDE})
#     install(TARGETS ${sourcename} DESTINATION bin)
# endforeach( sourcefile ${APP_SOURCES} )

file(GLOB SOURCES "*.cpp" "*.cc")

# Collect all header files in the project directory
file(GLOB HEADERS "*.hpp" "*.h")

# Create an executable target
add_executable(${PROJECT_NAME} ${SOURCES} ${HEADERS})
target_link_libraries( ${PROJECT_NAME} PUBLIC bfutils target_sys clish driver pthread)
