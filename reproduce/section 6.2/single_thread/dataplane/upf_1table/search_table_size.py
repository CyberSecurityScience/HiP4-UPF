
import re
import sys
import os


template_file = "settings_template.p4"
target_file = "settings.p4"

def try_value(value: int, template: str, write_only = False) -> bool :
    print('===================================================Trying', value)
    template = template.replace("<%VALUE%>", f"1024 * {value}")
    with open(target_file, 'w', encoding = 'utf-8') as fp :
        fp.write(template)
    if write_only :
        return True
    retval = os.system("bf-p4c upf.p4 --create-graphs --display-power-budget --log-hashes -g")
    return retval == 0

def main() :
    with open(template_file, 'r', encoding = 'utf-8') as fp :
        template = fp.read()

    val = 1
    lb = 0
    ub = 0
    while True :
        if try_value(val, template) :
            val *= 2
        else :
            lb = val // 2
            ub = val
            break

    while lb < ub :
        val = (lb + ub) // 2
        if val == lb :
            break
        if try_value(val, template) :
            lb = val
        else :
            ub = val - 1
    
    while True :
        if try_value(val, template) :
            break
        else :
            val -= 1

    try_value(val + 1, template, write_only = True)

if __name__ == '__main__' :
    main()
