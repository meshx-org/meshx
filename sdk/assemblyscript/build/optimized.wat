(module
 (type $i32_i32_=>_i32 (func (param i32 i32) (result i32)))
 (type $i32_i32_=>_none (func (param i32 i32)))
 (type $none_=>_none (func))
 (type $i32_i32_i32_i32_=>_i32 (func (param i32 i32 i32 i32) (result i32)))
 (type $i32_i32_i32_i32_=>_none (func (param i32 i32 i32 i32)))
 (type $i32_=>_i32 (func (param i32) (result i32)))
 (type $i32_i32_i32_=>_none (func (param i32 i32 i32)))
 (type $i32_=>_none (func (param i32)))
 (import "wasi_snapshot_preview1" "fd_write" (func $~lib/bindings/wasi_snapshot_preview1/fd_write (param i32 i32 i32 i32) (result i32)))
 (import "env" "abort" (func $~lib/builtins/abort (param i32 i32 i32 i32)))
 (global $~lib/rt/tlsf/ROOT (mut i32) (i32.const 0))
 (global $~lib/memory/__stack_pointer (mut i32) (i32.const 20812))
 (memory $0 1)
 (data (i32.const 1036) ",")
 (data (i32.const 1048) "\01\00\00\00\1a\00\00\00H\00e\00l\00l\00o\00,\00 \00W\00o\00r\00l\00d\00!")
 (data (i32.const 1116) ",")
 (data (i32.const 1128) "\01\00\00\00\0e\00\00\00S\00U\00C\00C\00E\00S\00S")
 (data (i32.const 1164) "\1c")
 (data (i32.const 1176) "\01\00\00\00\0c\00\00\00T\00O\00O\00B\00I\00G")
 (data (i32.const 1196) "\1c")
 (data (i32.const 1208) "\01\00\00\00\n\00\00\00A\00C\00C\00E\00S")
 (data (i32.const 1228) ",")
 (data (i32.const 1240) "\01\00\00\00\12\00\00\00A\00D\00D\00R\00I\00N\00U\00S\00E")
 (data (i32.const 1276) ",")
 (data (i32.const 1288) "\01\00\00\00\18\00\00\00A\00D\00D\00R\00N\00O\00T\00A\00V\00A\00I\00L")
 (data (i32.const 1324) ",")
 (data (i32.const 1336) "\01\00\00\00\16\00\00\00A\00F\00N\00O\00S\00U\00P\00P\00O\00R\00T")
 (data (i32.const 1372) "\1c")
 (data (i32.const 1384) "\01\00\00\00\n\00\00\00A\00G\00A\00I\00N")
 (data (i32.const 1404) ",")
 (data (i32.const 1416) "\01\00\00\00\0e\00\00\00A\00L\00R\00E\00A\00D\00Y")
 (data (i32.const 1452) "\1c")
 (data (i32.const 1464) "\01\00\00\00\08\00\00\00B\00A\00D\00F")
 (data (i32.const 1484) "\1c")
 (data (i32.const 1496) "\01\00\00\00\0c\00\00\00B\00A\00D\00M\00S\00G")
 (data (i32.const 1516) "\1c")
 (data (i32.const 1528) "\01\00\00\00\08\00\00\00B\00U\00S\00Y")
 (data (i32.const 1548) ",")
 (data (i32.const 1560) "\01\00\00\00\10\00\00\00C\00A\00N\00C\00E\00L\00E\00D")
 (data (i32.const 1596) "\1c")
 (data (i32.const 1608) "\01\00\00\00\n\00\00\00C\00H\00I\00L\00D")
 (data (i32.const 1628) ",")
 (data (i32.const 1640) "\01\00\00\00\16\00\00\00C\00O\00N\00N\00A\00B\00O\00R\00T\00E\00D")
 (data (i32.const 1676) ",")
 (data (i32.const 1688) "\01\00\00\00\16\00\00\00C\00O\00N\00N\00R\00E\00F\00U\00S\00E\00D")
 (data (i32.const 1724) ",")
 (data (i32.const 1736) "\01\00\00\00\12\00\00\00C\00O\00N\00N\00R\00E\00S\00E\00T")
 (data (i32.const 1772) "\1c")
 (data (i32.const 1784) "\01\00\00\00\0c\00\00\00D\00E\00A\00D\00L\00K")
 (data (i32.const 1804) ",")
 (data (i32.const 1816) "\01\00\00\00\16\00\00\00D\00E\00S\00T\00A\00D\00D\00R\00R\00E\00Q")
 (data (i32.const 1852) "\1c")
 (data (i32.const 1864) "\01\00\00\00\06\00\00\00D\00O\00M")
 (data (i32.const 1884) "\1c")
 (data (i32.const 1896) "\01\00\00\00\n\00\00\00D\00Q\00U\00O\00T")
 (data (i32.const 1916) "\1c")
 (data (i32.const 1928) "\01\00\00\00\n\00\00\00E\00X\00I\00S\00T")
 (data (i32.const 1948) "\1c")
 (data (i32.const 1960) "\01\00\00\00\n\00\00\00F\00A\00U\00L\00T")
 (data (i32.const 1980) "\1c")
 (data (i32.const 1992) "\01\00\00\00\08\00\00\00F\00B\00I\00G")
 (data (i32.const 2012) ",")
 (data (i32.const 2024) "\01\00\00\00\16\00\00\00H\00O\00S\00T\00U\00N\00R\00E\00A\00C\00H")
 (data (i32.const 2060) "\1c")
 (data (i32.const 2072) "\01\00\00\00\08\00\00\00I\00D\00R\00M")
 (data (i32.const 2092) "\1c")
 (data (i32.const 2104) "\01\00\00\00\n\00\00\00I\00L\00S\00E\00Q")
 (data (i32.const 2124) ",")
 (data (i32.const 2136) "\01\00\00\00\14\00\00\00I\00N\00P\00R\00O\00G\00R\00E\00S\00S")
 (data (i32.const 2172) "\1c")
 (data (i32.const 2184) "\01\00\00\00\08\00\00\00I\00N\00T\00R")
 (data (i32.const 2204) "\1c")
 (data (i32.const 2216) "\01\00\00\00\n\00\00\00I\00N\00V\00A\00L")
 (data (i32.const 2236) "\1c")
 (data (i32.const 2248) "\01\00\00\00\04\00\00\00I\00O")
 (data (i32.const 2268) "\1c")
 (data (i32.const 2280) "\01\00\00\00\0c\00\00\00I\00S\00C\00O\00N\00N")
 (data (i32.const 2300) "\1c")
 (data (i32.const 2312) "\01\00\00\00\n\00\00\00I\00S\00D\00I\00R")
 (data (i32.const 2332) "\1c")
 (data (i32.const 2344) "\01\00\00\00\08\00\00\00L\00O\00O\00P")
 (data (i32.const 2364) "\1c")
 (data (i32.const 2376) "\01\00\00\00\n\00\00\00M\00F\00I\00L\00E")
 (data (i32.const 2396) "\1c")
 (data (i32.const 2408) "\01\00\00\00\n\00\00\00M\00L\00I\00N\00K")
 (data (i32.const 2428) ",")
 (data (i32.const 2440) "\01\00\00\00\0e\00\00\00M\00S\00G\00S\00I\00Z\00E")
 (data (i32.const 2476) ",")
 (data (i32.const 2488) "\01\00\00\00\10\00\00\00M\00U\00L\00T\00I\00H\00O\00P")
 (data (i32.const 2524) ",")
 (data (i32.const 2536) "\01\00\00\00\16\00\00\00N\00A\00M\00E\00T\00O\00O\00L\00O\00N\00G")
 (data (i32.const 2572) ",")
 (data (i32.const 2584) "\01\00\00\00\0e\00\00\00N\00E\00T\00D\00O\00W\00N")
 (data (i32.const 2620) ",")
 (data (i32.const 2632) "\01\00\00\00\10\00\00\00N\00E\00T\00R\00E\00S\00E\00T")
 (data (i32.const 2668) ",")
 (data (i32.const 2680) "\01\00\00\00\14\00\00\00N\00E\00T\00U\00N\00R\00E\00A\00C\00H")
 (data (i32.const 2716) "\1c")
 (data (i32.const 2728) "\01\00\00\00\n\00\00\00N\00F\00I\00L\00E")
 (data (i32.const 2748) "\1c")
 (data (i32.const 2760) "\01\00\00\00\0c\00\00\00N\00O\00B\00U\00F\00S")
 (data (i32.const 2780) "\1c")
 (data (i32.const 2792) "\01\00\00\00\n\00\00\00N\00O\00D\00E\00V")
 (data (i32.const 2812) "\1c")
 (data (i32.const 2824) "\01\00\00\00\n\00\00\00N\00O\00E\00N\00T")
 (data (i32.const 2844) "\1c")
 (data (i32.const 2856) "\01\00\00\00\0c\00\00\00N\00O\00E\00X\00E\00C")
 (data (i32.const 2876) "\1c")
 (data (i32.const 2888) "\01\00\00\00\n\00\00\00N\00O\00L\00C\00K")
 (data (i32.const 2908) "\1c")
 (data (i32.const 2920) "\01\00\00\00\0c\00\00\00N\00O\00L\00I\00N\00K")
 (data (i32.const 2940) "\1c")
 (data (i32.const 2952) "\01\00\00\00\n\00\00\00N\00O\00M\00E\00M")
 (data (i32.const 2972) "\1c")
 (data (i32.const 2984) "\01\00\00\00\n\00\00\00N\00O\00M\00S\00G")
 (data (i32.const 3004) ",")
 (data (i32.const 3016) "\01\00\00\00\14\00\00\00N\00O\00P\00R\00O\00T\00O\00O\00P\00T")
 (data (i32.const 3052) "\1c")
 (data (i32.const 3064) "\01\00\00\00\n\00\00\00N\00O\00S\00P\00C")
 (data (i32.const 3084) "\1c")
 (data (i32.const 3096) "\01\00\00\00\n\00\00\00N\00O\00S\00Y\00S")
 (data (i32.const 3116) ",")
 (data (i32.const 3128) "\01\00\00\00\0e\00\00\00N\00O\00T\00C\00O\00N\00N")
 (data (i32.const 3164) "\1c")
 (data (i32.const 3176) "\01\00\00\00\0c\00\00\00N\00O\00T\00D\00I\00R")
 (data (i32.const 3196) ",")
 (data (i32.const 3208) "\01\00\00\00\10\00\00\00N\00O\00T\00E\00M\00P\00T\00Y")
 (data (i32.const 3244) ",")
 (data (i32.const 3256) "\01\00\00\00\1c\00\00\00N\00O\00T\00R\00E\00C\00O\00V\00E\00R\00A\00B\00L\00E")
 (data (i32.const 3292) ",")
 (data (i32.const 3304) "\01\00\00\00\0e\00\00\00N\00O\00T\00S\00O\00C\00K")
 (data (i32.const 3340) "\1c")
 (data (i32.const 3352) "\01\00\00\00\0c\00\00\00N\00O\00T\00S\00U\00P")
 (data (i32.const 3372) "\1c")
 (data (i32.const 3384) "\01\00\00\00\n\00\00\00N\00O\00T\00T\00Y")
 (data (i32.const 3404) "\1c")
 (data (i32.const 3416) "\01\00\00\00\08\00\00\00N\00X\00I\00O")
 (data (i32.const 3436) ",")
 (data (i32.const 3448) "\01\00\00\00\10\00\00\00O\00V\00E\00R\00F\00L\00O\00W")
 (data (i32.const 3484) ",")
 (data (i32.const 3496) "\01\00\00\00\12\00\00\00O\00W\00N\00E\00R\00D\00E\00A\00D")
 (data (i32.const 3532) "\1c")
 (data (i32.const 3544) "\01\00\00\00\08\00\00\00P\00E\00R\00M")
 (data (i32.const 3564) "\1c")
 (data (i32.const 3576) "\01\00\00\00\08\00\00\00P\00I\00P\00E")
 (data (i32.const 3596) "\1c")
 (data (i32.const 3608) "\01\00\00\00\n\00\00\00P\00R\00O\00T\00O")
 (data (i32.const 3628) ",")
 (data (i32.const 3640) "\01\00\00\00\1c\00\00\00P\00R\00O\00T\00O\00N\00O\00S\00U\00P\00P\00O\00R\00T")
 (data (i32.const 3676) ",")
 (data (i32.const 3688) "\01\00\00\00\12\00\00\00P\00R\00O\00T\00O\00T\00Y\00P\00E")
 (data (i32.const 3724) "\1c")
 (data (i32.const 3736) "\01\00\00\00\n\00\00\00R\00A\00N\00G\00E")
 (data (i32.const 3756) "\1c")
 (data (i32.const 3768) "\01\00\00\00\08\00\00\00R\00O\00F\00S")
 (data (i32.const 3788) "\1c")
 (data (i32.const 3800) "\01\00\00\00\n\00\00\00S\00P\00I\00P\00E")
 (data (i32.const 3820) "\1c")
 (data (i32.const 3832) "\01\00\00\00\08\00\00\00S\00R\00C\00H")
 (data (i32.const 3852) "\1c")
 (data (i32.const 3864) "\01\00\00\00\n\00\00\00S\00T\00A\00L\00E")
 (data (i32.const 3884) ",")
 (data (i32.const 3896) "\01\00\00\00\10\00\00\00T\00I\00M\00E\00D\00O\00U\00T")
 (data (i32.const 3932) "\1c")
 (data (i32.const 3944) "\01\00\00\00\0c\00\00\00T\00X\00T\00B\00S\00Y")
 (data (i32.const 3964) "\1c")
 (data (i32.const 3976) "\01\00\00\00\08\00\00\00X\00D\00E\00V")
 (data (i32.const 3996) ",")
 (data (i32.const 4008) "\01\00\00\00\14\00\00\00N\00O\00T\00C\00A\00P\00A\00B\00L\00E")
 (data (i32.const 4044) ",")
 (data (i32.const 4056) "\01\00\00\00\0e\00\00\00U\00N\00K\00N\00O\00W\00N")
 (data (i32.const 4092) "<")
 (data (i32.const 4104) "\01\00\00\00\1e\00\00\00~\00l\00i\00b\00/\00p\00r\00o\00c\00e\00s\00s\00.\00t\00s")
 (data (i32.const 4156) "<")
 (data (i32.const 4168) "\01\00\00\00\1e\00\00\00~\00l\00i\00b\00/\00r\00t\00/\00t\00l\00s\00f\00.\00t\00s")
 (data (i32.const 4220) "<")
 (data (i32.const 4232) "\01\00\00\00(\00\00\00A\00l\00l\00o\00c\00a\00t\00i\00o\00n\00 \00t\00o\00o\00 \00l\00a\00r\00g\00e")
 (data (i32.const 4284) "<")
 (data (i32.const 4296) "\01\00\00\00$\00\00\00U\00n\00p\00a\00i\00r\00e\00d\00 \00s\00u\00r\00r\00o\00g\00a\00t\00e")
 (data (i32.const 4348) ",")
 (data (i32.const 4360) "\01\00\00\00\1c\00\00\00~\00l\00i\00b\00/\00s\00t\00r\00i\00n\00g\00.\00t\00s")
 (data (i32.const 4396) "\1c")
 (data (i32.const 4408) "\01\00\00\00\02\00\00\00\n")
 (export "add" (func $assembly/index/add))
 (export "main" (func $assembly/index/main))
 (export "memory" (memory $0))
 (func $assembly/index/add (param $0 i32) (param $1 i32) (result i32)
  local.get $0
  local.get $1
  i32.add
 )
 (func $~lib/bindings/wasi_snapshot_preview1/errnoToString (param $0 i32) (result i32)
  block $break|0
   block $case76|0
    block $case75|0
     block $case74|0
      block $case73|0
       block $case72|0
        block $case71|0
         block $case70|0
          block $case69|0
           block $case68|0
            block $case67|0
             block $case66|0
              block $case65|0
               block $case64|0
                block $case63|0
                 block $case62|0
                  block $case61|0
                   block $case60|0
                    block $case59|0
                     block $case58|0
                      block $case57|0
                       block $case56|0
                        block $case55|0
                         block $case54|0
                          block $case53|0
                           block $case52|0
                            block $case51|0
                             block $case50|0
                              block $case49|0
                               block $case48|0
                                block $case47|0
                                 block $case46|0
                                  block $case45|0
                                   block $case44|0
                                    block $case43|0
                                     block $case42|0
                                      block $case41|0
                                       block $case40|0
                                        block $case39|0
                                         block $case38|0
                                          block $case37|0
                                           block $case36|0
                                            block $case35|0
                                             block $case34|0
                                              block $case33|0
                                               block $case32|0
                                                block $case31|0
                                                 block $case30|0
                                                  block $case29|0
                                                   block $case28|0
                                                    block $case27|0
                                                     block $case26|0
                                                      block $case25|0
                                                       block $case24|0
                                                        block $case23|0
                                                         block $case22|0
                                                          block $case21|0
                                                           block $case20|0
                                                            block $case19|0
                                                             block $case18|0
                                                              block $case17|0
                                                               block $case16|0
                                                                block $case15|0
                                                                 block $case14|0
                                                                  block $case13|0
                                                                   block $case12|0
                                                                    block $case11|0
                                                                     block $case10|0
                                                                      block $case9|0
                                                                       block $case8|0
                                                                        block $case7|0
                                                                         block $case6|0
                                                                          block $case5|0
                                                                           block $case4|0
                                                                            block $case3|0
                                                                             block $case2|0
                                                                              block $case1|0
                                                                               block $case0|0
                                                                                local.get $0
                                                                                i32.const 65535
                                                                                i32.and
                                                                                br_table $case0|0 $case1|0 $case2|0 $case3|0 $case4|0 $case5|0 $case6|0 $case7|0 $case8|0 $case9|0 $case10|0 $case11|0 $case12|0 $case13|0 $case14|0 $case15|0 $case16|0 $case17|0 $case18|0 $case19|0 $case20|0 $case21|0 $case22|0 $case23|0 $case24|0 $case25|0 $case26|0 $case27|0 $case28|0 $case29|0 $case30|0 $case31|0 $case32|0 $case33|0 $case34|0 $case35|0 $case36|0 $case37|0 $case38|0 $case39|0 $case40|0 $case41|0 $case42|0 $case43|0 $case44|0 $case45|0 $case46|0 $case47|0 $case48|0 $case49|0 $case50|0 $case51|0 $case52|0 $case53|0 $case54|0 $case55|0 $case56|0 $case57|0 $case58|0 $case59|0 $case60|0 $case61|0 $case62|0 $case63|0 $case64|0 $case65|0 $case66|0 $case67|0 $case68|0 $case69|0 $case70|0 $case71|0 $case72|0 $case73|0 $case74|0 $case75|0 $case76|0 $break|0
                                                                               end
                                                                               i32.const 1136
                                                                               return
                                                                              end
                                                                              i32.const 1184
                                                                              return
                                                                             end
                                                                             i32.const 1216
                                                                             return
                                                                            end
                                                                            i32.const 1248
                                                                            return
                                                                           end
                                                                           i32.const 1296
                                                                           return
                                                                          end
                                                                          i32.const 1344
                                                                          return
                                                                         end
                                                                         i32.const 1392
                                                                         return
                                                                        end
                                                                        i32.const 1424
                                                                        return
                                                                       end
                                                                       i32.const 1472
                                                                       return
                                                                      end
                                                                      i32.const 1504
                                                                      return
                                                                     end
                                                                     i32.const 1536
                                                                     return
                                                                    end
                                                                    i32.const 1568
                                                                    return
                                                                   end
                                                                   i32.const 1616
                                                                   return
                                                                  end
                                                                  i32.const 1648
                                                                  return
                                                                 end
                                                                 i32.const 1696
                                                                 return
                                                                end
                                                                i32.const 1744
                                                                return
                                                               end
                                                               i32.const 1792
                                                               return
                                                              end
                                                              i32.const 1824
                                                              return
                                                             end
                                                             i32.const 1872
                                                             return
                                                            end
                                                            i32.const 1904
                                                            return
                                                           end
                                                           i32.const 1936
                                                           return
                                                          end
                                                          i32.const 1968
                                                          return
                                                         end
                                                         i32.const 2000
                                                         return
                                                        end
                                                        i32.const 2032
                                                        return
                                                       end
                                                       i32.const 2080
                                                       return
                                                      end
                                                      i32.const 2112
                                                      return
                                                     end
                                                     i32.const 2144
                                                     return
                                                    end
                                                    i32.const 2192
                                                    return
                                                   end
                                                   i32.const 2224
                                                   return
                                                  end
                                                  i32.const 2256
                                                  return
                                                 end
                                                 i32.const 2288
                                                 return
                                                end
                                                i32.const 2320
                                                return
                                               end
                                               i32.const 2352
                                               return
                                              end
                                              i32.const 2384
                                              return
                                             end
                                             i32.const 2416
                                             return
                                            end
                                            i32.const 2448
                                            return
                                           end
                                           i32.const 2496
                                           return
                                          end
                                          i32.const 2544
                                          return
                                         end
                                         i32.const 2592
                                         return
                                        end
                                        i32.const 2640
                                        return
                                       end
                                       i32.const 2688
                                       return
                                      end
                                      i32.const 2736
                                      return
                                     end
                                     i32.const 2768
                                     return
                                    end
                                    i32.const 2800
                                    return
                                   end
                                   i32.const 2832
                                   return
                                  end
                                  i32.const 2864
                                  return
                                 end
                                 i32.const 2896
                                 return
                                end
                                i32.const 2928
                                return
                               end
                               i32.const 2960
                               return
                              end
                              i32.const 2992
                              return
                             end
                             i32.const 3024
                             return
                            end
                            i32.const 3072
                            return
                           end
                           i32.const 3104
                           return
                          end
                          i32.const 3136
                          return
                         end
                         i32.const 3184
                         return
                        end
                        i32.const 3216
                        return
                       end
                       i32.const 3264
                       return
                      end
                      i32.const 3312
                      return
                     end
                     i32.const 3360
                     return
                    end
                    i32.const 3392
                    return
                   end
                   i32.const 3424
                   return
                  end
                  i32.const 3456
                  return
                 end
                 i32.const 3504
                 return
                end
                i32.const 3552
                return
               end
               i32.const 3584
               return
              end
              i32.const 3616
              return
             end
             i32.const 3648
             return
            end
            i32.const 3696
            return
           end
           i32.const 3744
           return
          end
          i32.const 3776
          return
         end
         i32.const 3808
         return
        end
        i32.const 3840
        return
       end
       i32.const 3872
       return
      end
      i32.const 3904
      return
     end
     i32.const 3952
     return
    end
    i32.const 3984
    return
   end
   i32.const 4016
   return
  end
  i32.const 4064
 )
 (func $~lib/rt/tlsf/removeBlock (param $0 i32) (param $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  local.get $1
  i32.load
  local.tee $2
  i32.const 1
  i32.and
  i32.eqz
  if
   i32.const 0
   i32.const 4176
   i32.const 268
   i32.const 14
   call $~lib/builtins/abort
   unreachable
  end
  local.get $2
  i32.const -4
  i32.and
  local.tee $2
  i32.const 12
  i32.lt_u
  if
   i32.const 0
   i32.const 4176
   i32.const 270
   i32.const 14
   call $~lib/builtins/abort
   unreachable
  end
  local.get $2
  i32.const 256
  i32.lt_u
  if (result i32)
   local.get $2
   i32.const 4
   i32.shr_u
  else
   i32.const 31
   local.get $2
   i32.const 1073741820
   local.get $2
   i32.const 1073741820
   i32.lt_u
   select
   local.tee $2
   i32.clz
   i32.sub
   local.tee $4
   i32.const 7
   i32.sub
   local.set $3
   local.get $2
   local.get $4
   i32.const 4
   i32.sub
   i32.shr_u
   i32.const 16
   i32.xor
  end
  local.tee $2
  i32.const 16
  i32.lt_u
  local.get $3
  i32.const 23
  i32.lt_u
  i32.and
  i32.eqz
  if
   i32.const 0
   i32.const 4176
   i32.const 284
   i32.const 14
   call $~lib/builtins/abort
   unreachable
  end
  local.get $1
  i32.load offset=8
  local.set $5
  local.get $1
  i32.load offset=4
  local.tee $4
  if
   local.get $4
   local.get $5
   i32.store offset=8
  end
  local.get $5
  if
   local.get $5
   local.get $4
   i32.store offset=4
  end
  local.get $2
  local.get $3
  i32.const 4
  i32.shl
  i32.add
  i32.const 2
  i32.shl
  local.get $0
  i32.add
  i32.load offset=96
  local.get $1
  i32.eq
  if
   local.get $2
   local.get $3
   i32.const 4
   i32.shl
   i32.add
   i32.const 2
   i32.shl
   local.get $0
   i32.add
   local.get $5
   i32.store offset=96
   local.get $5
   i32.eqz
   if
    local.get $3
    i32.const 2
    i32.shl
    local.get $0
    i32.add
    local.tee $1
    i32.load offset=4
    i32.const -2
    local.get $2
    i32.rotl
    i32.and
    local.set $2
    local.get $1
    local.get $2
    i32.store offset=4
    local.get $2
    i32.eqz
    if
     local.get $0
     local.get $0
     i32.load
     i32.const -2
     local.get $3
     i32.rotl
     i32.and
     i32.store
    end
   end
  end
 )
 (func $~lib/rt/tlsf/insertBlock (param $0 i32) (param $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  (local $6 i32)
  local.get $1
  i32.eqz
  if
   i32.const 0
   i32.const 4176
   i32.const 201
   i32.const 14
   call $~lib/builtins/abort
   unreachable
  end
  local.get $1
  i32.load
  local.tee $3
  i32.const 1
  i32.and
  i32.eqz
  if
   i32.const 0
   i32.const 4176
   i32.const 203
   i32.const 14
   call $~lib/builtins/abort
   unreachable
  end
  local.get $1
  i32.const 4
  i32.add
  local.get $1
  i32.load
  i32.const -4
  i32.and
  i32.add
  local.tee $4
  i32.load
  local.tee $2
  i32.const 1
  i32.and
  if
   local.get $0
   local.get $4
   call $~lib/rt/tlsf/removeBlock
   local.get $1
   local.get $3
   i32.const 4
   i32.add
   local.get $2
   i32.const -4
   i32.and
   i32.add
   local.tee $3
   i32.store
   local.get $1
   i32.const 4
   i32.add
   local.get $1
   i32.load
   i32.const -4
   i32.and
   i32.add
   local.tee $4
   i32.load
   local.set $2
  end
  local.get $3
  i32.const 2
  i32.and
  if
   local.get $1
   i32.const 4
   i32.sub
   i32.load
   local.tee $1
   i32.load
   local.tee $6
   i32.const 1
   i32.and
   i32.eqz
   if
    i32.const 0
    i32.const 4176
    i32.const 221
    i32.const 16
    call $~lib/builtins/abort
    unreachable
   end
   local.get $0
   local.get $1
   call $~lib/rt/tlsf/removeBlock
   local.get $1
   local.get $6
   i32.const 4
   i32.add
   local.get $3
   i32.const -4
   i32.and
   i32.add
   local.tee $3
   i32.store
  end
  local.get $4
  local.get $2
  i32.const 2
  i32.or
  i32.store
  local.get $3
  i32.const -4
  i32.and
  local.tee $2
  i32.const 12
  i32.lt_u
  if
   i32.const 0
   i32.const 4176
   i32.const 233
   i32.const 14
   call $~lib/builtins/abort
   unreachable
  end
  local.get $2
  local.get $1
  i32.const 4
  i32.add
  i32.add
  local.get $4
  i32.ne
  if
   i32.const 0
   i32.const 4176
   i32.const 234
   i32.const 14
   call $~lib/builtins/abort
   unreachable
  end
  local.get $4
  i32.const 4
  i32.sub
  local.get $1
  i32.store
  local.get $2
  i32.const 256
  i32.lt_u
  if (result i32)
   local.get $2
   i32.const 4
   i32.shr_u
  else
   i32.const 31
   local.get $2
   i32.const 1073741820
   local.get $2
   i32.const 1073741820
   i32.lt_u
   select
   local.tee $2
   i32.clz
   i32.sub
   local.tee $3
   i32.const 7
   i32.sub
   local.set $5
   local.get $2
   local.get $3
   i32.const 4
   i32.sub
   i32.shr_u
   i32.const 16
   i32.xor
  end
  local.tee $2
  i32.const 16
  i32.lt_u
  local.get $5
  i32.const 23
  i32.lt_u
  i32.and
  i32.eqz
  if
   i32.const 0
   i32.const 4176
   i32.const 251
   i32.const 14
   call $~lib/builtins/abort
   unreachable
  end
  local.get $2
  local.get $5
  i32.const 4
  i32.shl
  i32.add
  i32.const 2
  i32.shl
  local.get $0
  i32.add
  i32.load offset=96
  local.set $3
  local.get $1
  i32.const 0
  i32.store offset=4
  local.get $1
  local.get $3
  i32.store offset=8
  local.get $3
  if
   local.get $3
   local.get $1
   i32.store offset=4
  end
  local.get $2
  local.get $5
  i32.const 4
  i32.shl
  i32.add
  i32.const 2
  i32.shl
  local.get $0
  i32.add
  local.get $1
  i32.store offset=96
  local.get $0
  local.get $0
  i32.load
  i32.const 1
  local.get $5
  i32.shl
  i32.or
  i32.store
  local.get $5
  i32.const 2
  i32.shl
  local.get $0
  i32.add
  local.tee $0
  local.get $0
  i32.load offset=4
  i32.const 1
  local.get $2
  i32.shl
  i32.or
  i32.store offset=4
 )
 (func $~lib/rt/tlsf/addMemory (param $0 i32) (param $1 i32) (param $2 i32)
  (local $3 i32)
  (local $4 i32)
  local.get $1
  local.get $2
  i32.gt_u
  if
   i32.const 0
   i32.const 4176
   i32.const 377
   i32.const 14
   call $~lib/builtins/abort
   unreachable
  end
  local.get $1
  i32.const 19
  i32.add
  i32.const -16
  i32.and
  i32.const 4
  i32.sub
  local.set $1
  local.get $0
  i32.load offset=1568
  local.tee $4
  if
   local.get $1
   local.get $4
   i32.const 4
   i32.add
   i32.lt_u
   if
    i32.const 0
    i32.const 4176
    i32.const 384
    i32.const 16
    call $~lib/builtins/abort
    unreachable
   end
   local.get $4
   local.get $1
   i32.const 16
   i32.sub
   i32.eq
   if
    local.get $4
    i32.load
    local.set $3
    local.get $1
    i32.const 16
    i32.sub
    local.set $1
   end
  else
   local.get $1
   local.get $0
   i32.const 1572
   i32.add
   i32.lt_u
   if
    i32.const 0
    i32.const 4176
    i32.const 397
    i32.const 5
    call $~lib/builtins/abort
    unreachable
   end
  end
  local.get $2
  i32.const -16
  i32.and
  local.get $1
  i32.sub
  local.tee $2
  i32.const 20
  i32.lt_u
  if
   return
  end
  local.get $1
  local.get $3
  i32.const 2
  i32.and
  local.get $2
  i32.const 8
  i32.sub
  local.tee $2
  i32.const 1
  i32.or
  i32.or
  i32.store
  local.get $1
  i32.const 0
  i32.store offset=4
  local.get $1
  i32.const 0
  i32.store offset=8
  local.get $2
  local.get $1
  i32.const 4
  i32.add
  i32.add
  local.tee $2
  i32.const 2
  i32.store
  local.get $0
  local.get $2
  i32.store offset=1568
  local.get $0
  local.get $1
  call $~lib/rt/tlsf/insertBlock
 )
 (func $~lib/rt/tlsf/initialize
  (local $0 i32)
  (local $1 i32)
  memory.size
  local.tee $1
  i32.const 0
  i32.le_s
  if (result i32)
   i32.const 1
   local.get $1
   i32.sub
   memory.grow
   i32.const 0
   i32.lt_s
  else
   i32.const 0
  end
  if
   unreachable
  end
  i32.const 20816
  i32.const 0
  i32.store
  i32.const 22384
  i32.const 0
  i32.store
  loop $for-loop|0
   local.get $0
   i32.const 23
   i32.lt_u
   if
    local.get $0
    i32.const 2
    i32.shl
    i32.const 20816
    i32.add
    i32.const 0
    i32.store offset=4
    i32.const 0
    local.set $1
    loop $for-loop|1
     local.get $1
     i32.const 16
     i32.lt_u
     if
      local.get $1
      local.get $0
      i32.const 4
      i32.shl
      i32.add
      i32.const 2
      i32.shl
      i32.const 20816
      i32.add
      i32.const 0
      i32.store offset=96
      local.get $1
      i32.const 1
      i32.add
      local.set $1
      br $for-loop|1
     end
    end
    local.get $0
    i32.const 1
    i32.add
    local.set $0
    br $for-loop|0
   end
  end
  i32.const 20816
  i32.const 22388
  memory.size
  i32.const 16
  i32.shl
  call $~lib/rt/tlsf/addMemory
  i32.const 20816
  global.set $~lib/rt/tlsf/ROOT
 )
 (func $~lib/rt/tlsf/searchBlock (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  (local $3 i32)
  local.get $1
  i32.const 256
  i32.lt_u
  if (result i32)
   local.get $1
   i32.const 4
   i32.shr_u
  else
   i32.const 31
   i32.const 1
   i32.const 27
   local.get $1
   i32.clz
   i32.sub
   i32.shl
   local.get $1
   i32.add
   i32.const 1
   i32.sub
   local.get $1
   local.get $1
   i32.const 536870910
   i32.lt_u
   select
   local.tee $1
   i32.clz
   i32.sub
   local.tee $3
   i32.const 7
   i32.sub
   local.set $2
   local.get $1
   local.get $3
   i32.const 4
   i32.sub
   i32.shr_u
   i32.const 16
   i32.xor
  end
  local.tee $1
  i32.const 16
  i32.lt_u
  local.get $2
  i32.const 23
  i32.lt_u
  i32.and
  i32.eqz
  if
   i32.const 0
   i32.const 4176
   i32.const 330
   i32.const 14
   call $~lib/builtins/abort
   unreachable
  end
  local.get $2
  i32.const 2
  i32.shl
  local.get $0
  i32.add
  i32.load offset=4
  i32.const -1
  local.get $1
  i32.shl
  i32.and
  local.tee $1
  if (result i32)
   local.get $1
   i32.ctz
   local.get $2
   i32.const 4
   i32.shl
   i32.add
   i32.const 2
   i32.shl
   local.get $0
   i32.add
   i32.load offset=96
  else
   local.get $0
   i32.load
   i32.const -1
   local.get $2
   i32.const 1
   i32.add
   i32.shl
   i32.and
   local.tee $1
   if (result i32)
    local.get $1
    i32.ctz
    local.tee $1
    i32.const 2
    i32.shl
    local.get $0
    i32.add
    i32.load offset=4
    local.tee $2
    i32.eqz
    if
     i32.const 0
     i32.const 4176
     i32.const 343
     i32.const 18
     call $~lib/builtins/abort
     unreachable
    end
    local.get $2
    i32.ctz
    local.get $1
    i32.const 4
    i32.shl
    i32.add
    i32.const 2
    i32.shl
    local.get $0
    i32.add
    i32.load offset=96
   else
    i32.const 0
   end
  end
 )
 (func $~lib/process/writeString (param $0 i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  (local $6 i32)
  (local $7 i32)
  block $break|0
   block $case4|0
    block $case3|0
     block $case2|0
      block $case1|0
       block $case0|0
        local.get $0
        i32.const 20
        i32.sub
        i32.load offset=16
        i32.const 1
        i32.shr_u
        local.tee $4
        br_table $case4|0 $case3|0 $case2|0 $case1|0 $case0|0 $break|0
       end
       local.get $0
       i32.load16_u offset=6
       local.tee $2
       i32.const 128
       i32.ge_u
       br_if $break|0
      end
      local.get $0
      i32.load16_u offset=4
      local.tee $3
      i32.const 128
      i32.ge_u
      br_if $break|0
     end
     local.get $0
     i32.load16_u offset=2
     local.tee $1
     i32.const 128
     i32.ge_u
     br_if $break|0
    end
    local.get $0
    i32.load16_u
    local.tee $5
    i32.const 128
    i32.ge_u
    br_if $break|0
    i32.const 1088
    i32.const 1096
    i32.store
    i32.const 1092
    local.get $4
    i32.store
    i32.const 1096
    local.get $1
    i32.const 8
    i32.shl
    local.get $5
    i32.or
    local.get $3
    i32.const 16
    i32.shl
    i32.or
    local.get $2
    i32.const 24
    i32.shl
    i32.or
    i32.store
    i32.const 1
    i32.const 1088
    i32.const 1
    i32.const 1100
    call $~lib/bindings/wasi_snapshot_preview1/fd_write
    local.tee $0
    i32.const 65535
    i32.and
    if
     local.get $0
     call $~lib/bindings/wasi_snapshot_preview1/errnoToString
     i32.const 4112
     i32.const 178
     i32.const 16
     call $~lib/builtins/abort
     unreachable
    end
   end
   return
  end
  local.get $0
  local.tee $1
  local.get $1
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.add
  local.set $2
  i32.const 0
  local.set $3
  loop $while-continue|0
   local.get $1
   local.get $2
   i32.lt_u
   if
    local.get $1
    i32.load16_u
    local.tee $5
    i32.const 128
    i32.lt_u
    if (result i32)
     local.get $3
     i32.const 1
     i32.add
    else
     local.get $5
     i32.const 2048
     i32.lt_u
     if (result i32)
      local.get $3
      i32.const 2
      i32.add
     else
      local.get $5
      i32.const 64512
      i32.and
      i32.const 55296
      i32.eq
      local.get $2
      local.get $1
      i32.const 2
      i32.add
      i32.gt_u
      i32.and
      if
       local.get $1
       i32.load16_u offset=2
       i32.const 64512
       i32.and
       i32.const 56320
       i32.eq
       if
        local.get $3
        i32.const 4
        i32.add
        local.set $3
        local.get $1
        i32.const 4
        i32.add
        local.set $1
        br $while-continue|0
       end
      end
      local.get $3
      i32.const 3
      i32.add
     end
    end
    local.set $3
    local.get $1
    i32.const 2
    i32.add
    local.set $1
    br $while-continue|0
   end
  end
  global.get $~lib/rt/tlsf/ROOT
  i32.eqz
  if
   call $~lib/rt/tlsf/initialize
  end
  local.get $0
  local.tee $2
  local.get $4
  i32.const 1
  i32.shl
  i32.add
  local.set $4
  global.get $~lib/rt/tlsf/ROOT
  local.set $0
  local.get $3
  i32.const 1073741820
  i32.gt_u
  if
   i32.const 4240
   i32.const 4176
   i32.const 458
   i32.const 29
   call $~lib/builtins/abort
   unreachable
  end
  local.get $0
  i32.const 12
  local.get $3
  i32.const 19
  i32.add
  i32.const -16
  i32.and
  i32.const 4
  i32.sub
  local.get $3
  i32.const 12
  i32.le_u
  select
  local.tee $5
  call $~lib/rt/tlsf/searchBlock
  local.tee $1
  i32.eqz
  if
   memory.size
   local.tee $1
   i32.const 4
   local.get $0
   i32.load offset=1568
   local.get $1
   i32.const 16
   i32.shl
   i32.const 4
   i32.sub
   i32.ne
   i32.shl
   i32.const 1
   i32.const 27
   local.get $5
   i32.clz
   i32.sub
   i32.shl
   i32.const 1
   i32.sub
   local.get $5
   i32.add
   local.get $5
   local.get $5
   i32.const 536870910
   i32.lt_u
   select
   i32.add
   i32.const 65535
   i32.add
   i32.const -65536
   i32.and
   i32.const 16
   i32.shr_u
   local.tee $6
   local.get $1
   local.get $6
   i32.gt_s
   select
   memory.grow
   i32.const 0
   i32.lt_s
   if
    local.get $6
    memory.grow
    i32.const 0
    i32.lt_s
    if
     unreachable
    end
   end
   local.get $0
   local.get $1
   i32.const 16
   i32.shl
   memory.size
   i32.const 16
   i32.shl
   call $~lib/rt/tlsf/addMemory
   local.get $0
   local.get $5
   call $~lib/rt/tlsf/searchBlock
   local.tee $1
   i32.eqz
   if
    i32.const 0
    i32.const 4176
    i32.const 496
    i32.const 16
    call $~lib/builtins/abort
    unreachable
   end
  end
  local.get $1
  i32.load
  i32.const -4
  i32.and
  local.get $5
  i32.lt_u
  if
   i32.const 0
   i32.const 4176
   i32.const 498
   i32.const 14
   call $~lib/builtins/abort
   unreachable
  end
  local.get $0
  local.get $1
  call $~lib/rt/tlsf/removeBlock
  local.get $1
  i32.load
  local.set $6
  local.get $5
  i32.const 4
  i32.add
  i32.const 15
  i32.and
  if
   i32.const 0
   i32.const 4176
   i32.const 357
   i32.const 14
   call $~lib/builtins/abort
   unreachable
  end
  local.get $6
  i32.const -4
  i32.and
  local.get $5
  i32.sub
  local.tee $7
  i32.const 16
  i32.ge_u
  if
   local.get $1
   local.get $6
   i32.const 2
   i32.and
   local.get $5
   i32.or
   i32.store
   local.get $5
   local.get $1
   i32.const 4
   i32.add
   i32.add
   local.tee $5
   local.get $7
   i32.const 4
   i32.sub
   i32.const 1
   i32.or
   i32.store
   local.get $0
   local.get $5
   call $~lib/rt/tlsf/insertBlock
  else
   local.get $1
   local.get $6
   i32.const -2
   i32.and
   i32.store
   local.get $1
   i32.const 4
   i32.add
   local.get $1
   i32.load
   i32.const -4
   i32.and
   i32.add
   local.tee $0
   local.get $0
   i32.load
   i32.const -3
   i32.and
   i32.store
  end
  local.get $1
  i32.const 4
  i32.add
  local.tee $0
  local.set $5
  local.get $0
  local.set $1
  loop $while-continue|00
   local.get $2
   local.get $4
   i32.lt_u
   if
    local.get $2
    i32.load16_u
    local.tee $6
    i32.const 128
    i32.lt_u
    if (result i32)
     local.get $1
     local.get $6
     i32.store8
     local.get $1
     i32.const 1
     i32.add
    else
     local.get $6
     i32.const 2048
     i32.lt_u
     if (result i32)
      local.get $1
      local.get $6
      i32.const 6
      i32.shr_u
      i32.const 192
      i32.or
      local.get $6
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.const 8
      i32.shl
      i32.or
      i32.store16
      local.get $1
      i32.const 2
      i32.add
     else
      local.get $6
      i32.const 56320
      i32.lt_u
      local.get $4
      local.get $2
      i32.const 2
      i32.add
      i32.gt_u
      i32.and
      local.get $6
      i32.const 63488
      i32.and
      i32.const 55296
      i32.eq
      i32.and
      if
       local.get $2
       i32.load16_u offset=2
       local.tee $7
       i32.const 64512
       i32.and
       i32.const 56320
       i32.eq
       if
        local.get $1
        local.get $6
        i32.const 1023
        i32.and
        i32.const 10
        i32.shl
        i32.const 65536
        i32.add
        local.get $7
        i32.const 1023
        i32.and
        i32.or
        local.tee $6
        i32.const 63
        i32.and
        i32.const 128
        i32.or
        i32.const 24
        i32.shl
        local.get $6
        i32.const 6
        i32.shr_u
        i32.const 63
        i32.and
        i32.const 128
        i32.or
        i32.const 16
        i32.shl
        i32.or
        local.get $6
        i32.const 12
        i32.shr_u
        i32.const 63
        i32.and
        i32.const 128
        i32.or
        i32.const 8
        i32.shl
        i32.or
        local.get $6
        i32.const 18
        i32.shr_u
        i32.const 240
        i32.or
        i32.or
        i32.store
        local.get $1
        i32.const 4
        i32.add
        local.set $1
        local.get $2
        i32.const 4
        i32.add
        local.set $2
        br $while-continue|00
       end
      end
      local.get $1
      local.get $6
      i32.const 12
      i32.shr_u
      i32.const 224
      i32.or
      local.get $6
      i32.const 6
      i32.shr_u
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.const 8
      i32.shl
      i32.or
      i32.store16
      local.get $1
      local.get $6
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.store8 offset=2
      local.get $1
      i32.const 3
      i32.add
     end
    end
    local.set $1
    local.get $2
    i32.const 2
    i32.add
    local.set $2
    br $while-continue|00
   end
  end
  local.get $1
  local.get $5
  i32.sub
  local.get $3
  i32.ne
  if
   i32.const 0
   i32.const 4112
   i32.const 184
   i32.const 3
   call $~lib/builtins/abort
   unreachable
  end
  i32.const 1088
  local.get $0
  i32.store
  i32.const 1092
  local.get $3
  i32.store
  i32.const 1
  i32.const 1088
  i32.const 1
  i32.const 1096
  call $~lib/bindings/wasi_snapshot_preview1/fd_write
  local.set $1
  local.get $0
  i32.const 20812
  i32.ge_u
  if
   global.get $~lib/rt/tlsf/ROOT
   i32.eqz
   if
    call $~lib/rt/tlsf/initialize
   end
   global.get $~lib/rt/tlsf/ROOT
   local.get $0
   i32.const 4
   i32.sub
   local.set $3
   local.get $0
   i32.const 15
   i32.and
   i32.const 1
   local.get $0
   select
   if (result i32)
    i32.const 1
   else
    local.get $3
    i32.load
    i32.const 1
    i32.and
   end
   if
    i32.const 0
    i32.const 4176
    i32.const 559
    i32.const 3
    call $~lib/builtins/abort
    unreachable
   end
   local.get $3
   local.get $3
   i32.load
   i32.const 1
   i32.or
   i32.store
   local.get $3
   call $~lib/rt/tlsf/insertBlock
  end
  local.get $1
  i32.const 65535
  i32.and
  if
   local.get $1
   call $~lib/bindings/wasi_snapshot_preview1/errnoToString
   i32.const 4112
   i32.const 189
   i32.const 12
   call $~lib/builtins/abort
   unreachable
  end
 )
 (func $assembly/index/main
  (local $0 i32)
  global.get $~lib/memory/__stack_pointer
  i32.const 4
  i32.sub
  global.set $~lib/memory/__stack_pointer
  block $folding-inner0
   global.get $~lib/memory/__stack_pointer
   i32.const 4428
   i32.lt_s
   br_if $folding-inner0
   global.get $~lib/memory/__stack_pointer
   local.tee $0
   i32.const 0
   i32.store
   local.get $0
   i32.const 1056
   i32.store
   local.get $0
   i32.const 4
   i32.sub
   global.set $~lib/memory/__stack_pointer
   global.get $~lib/memory/__stack_pointer
   i32.const 4428
   i32.lt_s
   br_if $folding-inner0
   global.get $~lib/memory/__stack_pointer
   i32.const 0
   i32.store
   i32.const 1056
   call $~lib/process/writeString
   global.get $~lib/memory/__stack_pointer
   i32.const 4416
   i32.store
   i32.const 4416
   call $~lib/process/writeString
   global.get $~lib/memory/__stack_pointer
   i32.const 4
   i32.add
   global.set $~lib/memory/__stack_pointer
   global.get $~lib/memory/__stack_pointer
   i32.const 4
   i32.add
   global.set $~lib/memory/__stack_pointer
   return
  end
  i32.const 20832
  i32.const 20880
  i32.const 1
  i32.const 1
  call $~lib/builtins/abort
  unreachable
 )
)
