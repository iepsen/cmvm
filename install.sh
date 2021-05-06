#!/bin/bash

OUTPUT_PREFIX="[cmvm]"

if [[ $SHELL == "/bin/zsh" ]]
then
  SOURCE_FILE="$HOME/.zshrc"
else
  SOURCE_FILE="$HOME/.bash_profile"
fi

print_message() {
  echo $OUTPUT_PREFIX $1
}

create_symbolic_link() {
  if [[ -L /usr/local/bin/cmvm ]]
  then
    unlink /usr/local/bin/cmvm  
  fi
  ln -s ${PWD}/cmvm /usr/local/bin/cmvm  
}

bootstrap() {
  print_message "Creating basic folders and files structure to manage CMake versions..."
  rm -rf $HOME/.cmvm
  mkdir -p $HOME/.cmvm/{bin,versions}
  cmvm_source=$HOME/.cmvm/cmvm_source

  touch $cmvm_source
  echo "# cmvm" >> $cmvm_source
  echo export PATH="\$HOME/.cmvm/current/bin:\$PATH" >> $cmvm_source

  echo "# cmvm" >> $SOURCE_FILE
  echo "source $cmvm_source" >> $SOURCE_FILE

  source $cmvm_source
}

setup() {
  print_message "Setting up shell scripts..."
  
  cmvm_url=https://raw.githubusercontent.com/iepsen/cmvm/master/cmvm

  cd $HOME/.cmvm/bin
  curl -sS ${cmvm_url} > cmvm && \
  chmod +x cmvm && \
  create_symbolic_link

  print_message "Done! You can now use cmvm to manage CMake versions."
  echo ""

  cmvm
}

bootstrap
setup