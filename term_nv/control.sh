#!/bin/bash
faStabilo='\033[7m'
fcRouge='\033[31m'
fcJaune='\033[33;1m'
fcCyan='\033[36m'
fcGreen='\033[32m'
fcBleu='\033[34m'
fcNoir='\033[0;0m'

faGras='\033[1m'

lign='\033[1m\n'
=========================
# function  menu
#=========================

f_cls() {

reset > /dev/null
    echo -en '\033[1;1H'
    echo -en '\033]11;#000000\007'
    echo -en '\033]10;#FFFFFF\007'
}

f_pause(){
    echo -en '\033[0;0m'
     echo -en $faStabilo$fcRouge'Press[Enter] key to continue'
    tput civis     # curseur invisible
    read -s -n 1
    echo -en '\033[0;0m\n'
}





trap cleanup SIGHUP SIGTERM SIGINT
# resize
printf '\e[8;'25';'80't'
f_cls

echo -e $faGras$fcJaune'cargo update'
echo -e $lign
cargo update
echo -e $lign
cargo clean
echo -e $lign
f_pause



echo -e $faGras$fcJaune'cargo outdated'
echo -e $lign
cargo outdated
echo -e $lign
f_pause

echo -e $faGras$fcJaune'cargo audit'
echo -e $lign
cargo audit
echo -e $lign
f_pause

echo -e $faGras$fcJaune'cargo deny check'
echo -e $lign
cargo deny check
echo -e $lign
f_pause
exit 5;
