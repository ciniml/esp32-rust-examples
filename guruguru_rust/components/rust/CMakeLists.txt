idf_component_register(SRCS dummy.c)

add_custom_command(TARGET ${COMPONENT_LIB}
                  POST_BUILD
                  COMMAND ${CMAKE_CURRENT_LIST_DIR}/build.sh ${CMAKE_CURRENT_LIST_DIR}
                  COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_LIST_DIR}/target/xtensa-esp32-none-elf/debug/librust_main.a librust.a
                  COMMENT "Building rust sources with cargo xbuild...")
