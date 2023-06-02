/* eslint-disable no-unused-vars */
import { DeleteIcon, RepeatIcon } from "@chakra-ui/icons"
import {
  CardBody,
  CardHeader,
  Flex,
  Heading,
  IconButton,
  Stack,
  Text
} from "@chakra-ui/react"
import React from "react"
import Address from "./Address"
import Balance from "./Balance"
import TransferForm from "./TransferForm"

interface AddressProps {
  balance: bigint
  chain: string
  address: string
  network: string
  handlerAddressRemove: (network: string, chain: string) => void
  handleTransfer: (from: string, to: string, amount: bigint) => Promise<void>
  handleTopup?: (from: string, to: string, amount: bigint) => Promise<void>
  handleBalance: () => Promise<void>
  loading: boolean
}

export const Chain: React.FC<AddressProps> = ({
  balance,
  chain,
  address,
  network,
  handlerAddressRemove,
  handleTransfer,
  handleBalance,
  handleTopup,
  loading,
  ...rest
}) => {
  return (
    <Stack
      direction="column"
      borderWidth="1px"
      borderRadius="lg"
      overflow="hidden"
      {...rest}
    >
      <CardHeader pb={0}>
        <Stack direction="row" justify="space-between" align="center">
          <Flex flex={5}>
            <Heading size="xs">{chain}</Heading>
          </Flex>
          <Flex flex={5}>
            <Text>{network}</Text>
          </Flex>
          <Stack direction="row" justify="end" align="center" flex={2}>
            <IconButton
              aria-label="Refresh"
              icon={<RepeatIcon />}
              color="green"
              onClick={handleBalance}
            />
            <IconButton
              aria-label="Remove"
              onClick={() => handlerAddressRemove(chain, network)}
              icon={<DeleteIcon />}
              color="red"
            />
          </Stack>
        </Stack>
      </CardHeader>
      <CardBody marginTop={0}>
        <Stack>
          <Stack direction="row" justify="space-between" align="center">
            <Address address={address} flex={9} />
            <Balance
              amount={balance}
              chain={chain}
              loading={loading}
              flex={3}
            />
          </Stack>
          <TransferForm
            address={address}
            loading={loading}
            title={`Send ${chain}`}
            handleTransfer={handleTransfer}
          />
          {handleTopup && (
            <TransferForm
              address={address}
              loading={loading}
              title="Topup"
              handleTransfer={handleTopup}
            />
          )}
        </Stack>
      </CardBody>
    </Stack>
  )
}
