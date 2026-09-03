#!/bin/bash
faStabilo='\033[7m'
fcRouge='\033[31m'
fcJaune='\033[33;1m'
fcCyan='\033[36m'
fcGreen='\033[32m'
fcBleu='\033[34m'
fcNoir='\033[0;0m'

faGras='\033[1m'

#=========================
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
    echo -en '\033[0;0m'
}

f_dsplyPos(){ #commande de positionnement    lines + coln + couleur + text
    echo -en '\033[0;0m'
    let lig=$1
    let col=$2
    echo -en '\033['$lig';'$col'f'$3$4

}
f_readPos() {    #commande de positionnement    lines + coln + text
    echo -en '\033[0;0m'
    let lig=$1
    let col=$2
    let colR=$2+${#3}+1  # si on doit coller faire  $2+${#3}
    echo -en '\033['$lig';'$col'f'$fdVert$faGras$fcBlanc$3
    echo -en '\033[0;0m'
    tput cnorm    # curseur visible
     echo -en '\033['$lig';'$colR'f'$faGras$fcGreen
    read
    tput civis     # curseur invisible
    echo -en '\033[0;0m'
}

# resize
printf '\e[8;'25';'80't'

envCPP="1"
envRUST="5"
libRUST="6"
editing="30"
PROJECT="CONSOLE-RUST"
LIBPROJECT="$HOME/Zrust/term_nv/"


choix=""

#@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
# Vérifier que cargo est installé
if ! command -v cargo &> /dev/null; then
    echo -e "${fcRouge}Erreur : cargo n'est pas installé.${fcNoir}"
    exit 1
fi


# Fonction de nettoyage
cleanup() {
    tput sgr0 # Réinitialise les couleurs
    tput cnorm            # Affiche le curseur
    if [ -d "$LIBPROJECT/target" ]; then
        rm -rf "$LIBPROJECT/target"
    fi
    exit 0
}

# Intercepter les signaux de fermeture
trap cleanup SIGHUP SIGTERM SIGINT
#@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@

while [ "$choix" != "99" ]
do
    cd $LIBPROJECT
    f_cls
    f_dsplyPos  1  24 $faGras$fcJaune'Project: '$faGras$fcCyan$PROJECT

    f_dsplyPos  2  24 $faGras$fcJaune'------------Outils----------------'
    f_dsplyPos  3  20 $faGras$fcRouge'  1'; f_dsplyPos  3  24 $faGras$fcGreen 'Browser'


    f_dsplyPos  5  24 $faGras$fcJaune'------------compile Rust----------------'
    f_dsplyPos  7  20 $faGras$fcRouge' 10'; f_dsplyPos  7  24 $faGras$fcGreen 'TermNV'
    f_dsplyPos  8  20 $faGras$fcRouge' 22'; f_dsplyPos  8  24 $faGras$fcGreen ' move TermNV to ~/Terminal'
    f_dsplyPos  9  24 $faGras$fcJaune '----------------------------------------'



    f_dsplyPos 13  20 $faGras$fcRouge'60.'; f_dsplyPos 13  24 $faGras$fcCyan  'EDIT FILE'
    f_dsplyPos 15  20 $faGras$fcRouge'77.'; f_dsplyPos 15  24 $faGras$fcCyan  'cargo clean'

    f_dsplyPos 17  20 $faGras$fcRouge'88.'; f_dsplyPos 17  24 $faGras$fcGreen 'Console'

    f_dsplyPos 19  20 $faGras$fcRouge'99.'; f_dsplyPos 19 24 $faGras$fcRouge  'Exit'

    f_dsplyPos 20  24 $faGras$fcBleu '----------------------------------------'
    f_readPos  24  20  'Votre choix  :'; choix=$REPLY;
    
    # Recherche de caractères non numériques dans les arguments.
    if echo $choix | tr -d [:blank:] | tr -d [:digit:] | grep . &> /dev/null; then
        f_readPos 24 70  'erreur de saisie Enter'
    else

         case "$choix" in



#thunar

				1) 
					thunar $HOME/Zrust/
				;;



#bin"

        10)
            $HOME/.Terminal/dispatch.sh $envRUST $LIBPROJECT   "TermNV" "Console"
        ;;
        22) rm -f $HOME/.Terminal/TermNV
            mv TermNV $HOME/.Terminal/TermNV
        ;;


#project
        60)
            $HOME/.Terminal/ProjectNVIM.sh  $PROJECT $LIBPROJECT
        ;;


#?clear
        77)
        		cargo clean
        		f_pause
						rm -rf target/
						rm -rf ~/.local/share/nvim/lsp_log/rust-analyzer*
        ;;

#console

        88)
            $HOME/.Terminal/console.sh $LIBPROJECT
        ;;



# QUIT
        99)
            break
        ;;

    esac
    fi # fintest option

printf '\e[8;'25';'80't'

done

cleanup
